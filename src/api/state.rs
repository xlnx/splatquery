use std::sync::Arc;

use axum::extract::FromRef;
use chrono::Duration;

use crate::{action::ActionAgentMap, database::Database, splatnet::SplatNet};

use super::{auth::AuthAgentMap, jwt};

#[derive(Clone)]
pub struct AppState(pub Arc<InnerAppState>);

pub struct InnerAppState {
  pub splatnet: Arc<SplatNet>,
  pub db: Database,
  pub jwt: jwt::Agent,
  pub auths: Arc<AuthAgentMap>,
  pub actions: Arc<ActionAgentMap>,
  pub auth_expiration: Duration,
}

impl FromRef<AppState> for Arc<SplatNet> {
  fn from_ref(input: &AppState) -> Self {
    input.0.splatnet.clone()
  }
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
