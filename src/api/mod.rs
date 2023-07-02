use axum::{
  async_trait,
  extract::{FromRef, FromRequestParts, State},
};
use axum_auth::AuthBearer;
use http::{request::Parts, StatusCode};
use serde::{Deserialize, Serialize};

pub mod action;
pub mod auth;
pub mod config;
pub mod jwt;
pub mod query;
pub mod state;

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
  pub agent: String,
  #[serde(flatten)]
  pub auth: auth::AuthUserInfo,
}

pub struct User(pub UserInfo);

#[async_trait]
impl<S> FromRequestParts<S> for User
where
  S: Send + Sync,
  jwt::Agent: FromRef<S>,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let AuthBearer(token) = AuthBearer::from_request_parts(parts, state).await?;
    let State(jwt) = State::<jwt::Agent>::from_request_parts(parts, state)
      .await
      .unwrap();
    let info = jwt.decode(&token).map_err(|err| {
      log::debug!("invalid access token: [{}], error: [{:?}]", token, err);
      (StatusCode::UNAUTHORIZED, "invalid access token")
    })?;
    Ok(User(info))
  }
}
