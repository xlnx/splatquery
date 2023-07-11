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
use tower_http::cors::{Any, CorsLayer};

use splatquery::{
  api::{
    self,
    config::Config,
    jwt,
    state::{AppState, InnerAppState},
  },
  database::Database,
  splatnet::SplatNetAgent,
  BoxError, Error, Result,
};

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

  // prepare action agents
  let actions = config.actions.collect()?;

  // prepare auth agents
  let auths = config.auth.agents.collect()?;

  // prepare database agent
  let db = Database::new_from_file(config.database.path)?;

  // prepare jwt agent
  let jwt = jwt::Agent::new(config.auth.token.algorithm, &config.auth.token.secret);
  let auth_expiration = Duration::days(config.auth.token.expire_days);

  // prepare splatnet agent
  let splatnet = SplatNetAgent::new(db.clone(), actions.clone(), config.splatnet)
    .watch()
    .map_err(|err| Error::InternalServerError(err));

  // make app state
  let state = AppState(Arc::new(InnerAppState {
    db,
    jwt,
    actions,
    auths,
    auth_expiration,
  }));

  let app = Router::new()
    .route("/status", get(|| async { Json(json!({"status": "ok"})) }))
    // query apis
    .route("/query/new", post(api::query::create))
    .route("/query/list", get(api::query::list))
    .route("/query/update", post(api::query::update))
    .route("/query/delete", post(api::query::delete))
    // action apis
    .route("/action/:agent/toggle", post(api::action::toggle))
    .route("/action/list", get(api::action::list))
    // auth apis
    .route("/auth/:agent", post(api::auth::oauth2));

  #[cfg(feature = "webpush")]
  use api::action::webpush;
  #[cfg(feature = "webpush")]
  let app = app
    .route("/action/webpush/subscribe", post(webpush::subscribe))
    .route("/action/webpush/dismiss", post(webpush::dismiss));

  let app = app.with_state(state);

  // add cors layer to the top
  let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers(Any)
    .expose_headers([AUTHORIZATION]);

  let cors = if config.http.allow_origins.is_empty() {
    cors.allow_origin(Any)
  } else {
    let iter = config
      .http
      .allow_origins
      .iter()
      .map(|e| e.parse::<HeaderValue>());
    let origins: Vec<_> = itertools::process_results(iter, |iter| iter.collect())?;
    cors.allow_origin(origins)
  };

  let app = app.layer(cors);

  let tls = RustlsConfig::from_pem_file(config.http.tls.pem, config.http.tls.key).await?;
  let addr = SocketAddr::from(([0, 0, 0, 0], config.http.port));
  log::info!("listening on {}", addr);

  let server = axum_server::bind_rustls(addr, tls)
    .serve(app.into_make_service())
    .map_err(|err| Error::InternalServerError(Box::new(err)));

  futures::try_join!(splatnet, server)?;

  Ok(())
}
