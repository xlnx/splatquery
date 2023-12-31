use std::{fs::File, io::BufReader, net::SocketAddr, sync::Arc};

use axum::{
  routing::{get, post},
  Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use chrono::Duration;
use futures::TryFutureExt;
use http::{header::AUTHORIZATION, HeaderValue, Method};
use serde_json::json;
use tower_http::{
  cors::{Any, CorsLayer},
  services::ServeDir,
};

#[cfg(feature = "renderer")]
use splatquery::renderer::Renderer;
use splatquery::{
  action::{ActionContext, ActionManager},
  api::{
    self,
    config::Config,
    jwt,
    state::{AppState, InnerAppState},
  },
  splatnet::SplatNetAgent,
  BoxError, Error,
};

fn cors<S>(app: Router<S>, allow_origins: &[String]) -> Result<Router<S>, BoxError>
where
  S: Clone + Send + Sync + 'static,
{
  // add cors layer to the top
  let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers(Any)
    .expose_headers([AUTHORIZATION]);

  let cors = if allow_origins.is_empty() {
    cors.allow_origin(Any)
  } else {
    let iter = allow_origins.iter().map(|e| e.parse::<HeaderValue>());
    let origins: Vec<_> = itertools::process_results(iter, |iter| iter.collect())?;
    cors.allow_origin(origins)
  };

  Ok(app.layer(cors))
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
  std::env::set_var(
    "RUST_LOG",
    "info,
     rustls=error,
     splatquery=debug",
  );
  env_logger::init();

  let path = std::env::args().skip(1).next().unwrap();
  let file = File::open(path)?;
  let reader = BufReader::new(file);
  let config: Config = serde_json::from_reader(reader)?;

  // prepare database agent
  let db = config.database.collect()?;

  #[cfg(feature = "renderer")]
  let renderer = Renderer::new(config.renderer)?;

  // prepare action agents
  let actions = ActionManager::new(
    ActionContext {
      database: db.clone(),
      #[cfg(feature = "renderer")]
      renderer: renderer.clone(),
      #[cfg(feature = "renderer")]
      image_url: format!(
        "https://{}:{}/_/image",
        config.http.cname.unwrap(),
        config.http.port,
      ),
    },
    config.actions.collect()?,
  );

  // prepare auth agents
  let auths = config.auth.agents.collect()?;

  // prepare jwt agent
  let jwt = jwt::Agent::new(config.auth.token.algorithm, &config.auth.token.secret);
  let auth_expiration = Duration::days(config.auth.token.expire_days);

  #[cfg(feature = "api-geoip2")]
  let geoip2 = if let Some(conf) = config.geoip2 {
    Some(conf.collect()?)
  } else {
    None
  };

  // prepare splatnet agent
  let splatnet = SplatNetAgent::new(actions.clone(), config.splatnet)
    .watch()
    .map_err(|err| Error::InternalServerError(err));

  // make app state
  let state = AppState(Arc::new(InnerAppState {
    db,
    jwt,
    actions,
    auths,
    auth_expiration,
    #[cfg(feature = "api-geoip2")]
    geoip2,
  }));

  let app = Router::new()
    .route("/status", get(|| async { Json(json!({"status": "ok"})) }))
    // user apis
    .route("/user/list", get(api::user::list))
    .route("/user/update", post(api::user::update))
    // query apis
    .route("/query/new", post(api::query::create))
    .route("/query/list", get(api::query::list))
    .route("/query/update", post(api::query::update))
    .route("/query/delete", post(api::query::delete))
    // action apis
    .route("/action/:agent/toggle", post(api::action::toggle))
    .route("/action/:agent/test", post(api::action::test))
    .route("/action/list", get(api::action::list))
    .route("/action/delete", post(api::action::delete))
    // auth apis
    .route("/auth/:agent", post(api::auth::oauth2));

  #[cfg(feature = "webpush")]
  let app = app.route(
    "/action/webpush/subscribe",
    post(api::action::webpush::subscribe),
  );

  // add cors layer to the top
  let app = cors(app.with_state(state), &config.http.allow_origins)?;

  #[cfg(feature = "renderer")]
  let app = app.nest_service("/_/image", ServeDir::new(renderer.out_dir()));

  let tls = RustlsConfig::from_pem_file(config.http.tls.pem, config.http.tls.key).await?;
  let addr = SocketAddr::from(([0, 0, 0, 0], config.http.port));
  log::info!("listening on {}", addr);

  let server = axum_server::bind_rustls(addr, tls)
    .serve(app.into_make_service_with_connect_info::<SocketAddr>())
    .map_err(|err| Error::InternalServerError(Box::new(err)));

  futures::try_join!(splatnet, server)?;

  Ok(())
}
