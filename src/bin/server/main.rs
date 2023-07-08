use std::{fs::File, io::BufReader, net::SocketAddr, sync::Arc};

use axum::{
  routing::{get, post},
  Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use chrono::Duration;
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
  splatnet::SplatNet,
  BoxError, Result,
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

  // prepare auth agents
  let auths = config.auth.agents.collect()?;

  // prepare action agents
  let actions = config.actions.collect()?;

  // prepare database agent
  let db = Database::new_from_file(config.database.path)?;

  // prepare jwt agent
  let jwt = jwt::Agent::new(config.auth.token.algorithm, &config.auth.token.secret);
  let auth_expiration = Duration::days(config.auth.token.expire_days);

  // prepare splatnet agent
  let splatnet = SplatNet::new(db.clone(), actions.clone(), config.splatnet);
  tokio::spawn(splatnet.clone().watch());

  // make app state
  let state = AppState(Arc::new(InnerAppState {
    splatnet,
    db,
    jwt,
    auths,
    actions,
    auth_expiration,
  }));

  let app = Router::new()
    .route("/status", get(|| async { Json(json!({"status": "ok"})) }))
    .route("/action/:agent/update", post(api::action::update))
    .route("/query/new", post(api::query::create))
    .route("/query/list", get(api::query::list))
    .route("/query/update", post(api::query::update))
    .route("/query/delete", post(api::query::delete))
    .route("/auth/:agent", post(api::auth::oauth2))
    .with_state(state);

  // add cors layer to the top
  let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers(Any)
    .expose_headers([AUTHORIZATION])
    .allow_origin([
      "https://splatquery.koishi.top".parse::<HeaderValue>()?,
      "http://localhost:5173".parse::<HeaderValue>()?,
      "http://localhost:8080".parse::<HeaderValue>()?,
      "http://localhost:8000".parse::<HeaderValue>()?,
    ]);

  let app = app.layer(cors);

  let tls = RustlsConfig::from_pem_file(config.cert.pem, config.cert.key).await?;
  let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
  log::info!("listening on {}", addr);

  axum_server::bind_rustls(addr, tls)
    .serve(app.into_make_service())
    .await?;

  Ok(())
}
