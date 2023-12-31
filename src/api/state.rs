use std::sync::Arc;

use axum::extract::FromRef;
use chrono::Duration;

use crate::{action::ActionManager, database::Database};

use super::{auth::AuthAgentMap, geoip2::GeoIp2, jwt};

#[derive(Clone)]
pub struct AppState(pub Arc<InnerAppState>);

pub struct InnerAppState {
  pub db: Database,
  pub jwt: jwt::Agent,
  pub actions: ActionManager,
  pub auths: Arc<AuthAgentMap>,
  pub auth_expiration: Duration,
  #[cfg(feature = "api-geoip2")]
  pub geoip2: Option<GeoIp2>,
}

impl FromRef<AppState> for Database {
  fn from_ref(input: &AppState) -> Self {
    input.0.db.clone()
  }
}

impl FromRef<AppState> for jwt::Agent {
  fn from_ref(input: &AppState) -> Self {
    input.0.jwt.clone()
  }
}

impl FromRef<AppState> for Arc<AuthAgentMap> {
  fn from_ref(input: &AppState) -> Self {
    input.0.auths.clone()
  }
}
