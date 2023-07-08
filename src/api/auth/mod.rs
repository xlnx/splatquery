use std::{collections::HashMap, sync::Arc};

use axum::{
  async_trait,
  extract::{Path, State},
  response::{AppendHeaders, IntoResponse},
  Json,
};
use http::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};

use crate::{
  api::UserInfo,
  database::user::{CreateUser, CreateUserRequest},
  Error, Result,
};

use super::state::{AppState, InnerAppState};

#[cfg(feature = "api-auth-google")]
pub mod google;

#[derive(Deserialize, Debug)]
pub struct AuthRequest {
  pub code: String,
  pub redirect_uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthUserInfo {
  pub id: String,
  pub name: Option<String>,
  pub email: Option<String>,
  pub picture: Option<String>,
}

pub type AuthAgentMap = HashMap<&'static str, Arc<dyn AuthAgent>>;

#[async_trait]
pub trait AuthAgent: Send + Sync {
  async fn oauth2(&self, request: &AuthRequest) -> Result<AuthUserInfo>;
}

pub async fn oauth2(
  Path(agent_type): Path<String>,
  State(state): State<AppState>,
  Json(request): Json<AuthRequest>,
) -> Result<impl IntoResponse> {
  let InnerAppState {
    auths,
    db,
    jwt,
    auth_expiration,
    ..
  } = state.0.as_ref();

  // find the specified agent
  let agent = auths
    .get(agent_type.as_str())
    .ok_or_else(|| Error::InvalidParameter("agent_type", agent_type.clone()))?;
  log::debug!("incoming auth request: [{:?}]", request);

  // send oauth2 request to auth server
  let auth = agent.oauth2(&request).await?;

  // store userinfo to db
  let ok = db.get()?.create_user(CreateUserRequest {
    auth_agent: &agent_type,
    auth_uid: &auth.id,
    name: auth.name.as_deref(),
    email: auth.email.as_deref(),
    picture: auth.picture.as_deref(),
  })?;

  if ok {
    log::debug!("user created: [{:?}]", &auth);
  } else {
    log::debug!("user already exists: [{:?}]", (&agent_type, &auth.id));
  }

  // sign our jwt
  let jwt = jwt.encode(
    &UserInfo {
      agent: agent_type,
      auth,
    },
    &auth_expiration,
  )?;
  log::debug!("signed auth request: [{:?}], jwt: [{}]", request, jwt);

  // emit jwt
  Ok(AppendHeaders([(
    AUTHORIZATION,
    String::from("Bearer ") + &jwt,
  )]))
}
