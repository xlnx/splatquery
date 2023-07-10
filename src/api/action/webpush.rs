use axum::{extract::State, response::IntoResponse, Json};

use crate::{
  action::webpush::{
    WebPushDismiss, WebPushDismissRequest, WebPushSubscribe, WebPushSubscribeRequest,
  },
  api::{
    state::{AppState, InnerAppState},
    User,
  },
  database::user::{LookupUser, LookupUserRequest},
  Result,
};

pub async fn subscribe(
  User(user): User,
  State(state): State<AppState>,
  Json(request): Json<WebPushSubscribeRequest>,
) -> Result<impl IntoResponse> {
  let InnerAppState { db, .. } = state.0.as_ref();

  let conn = db.get()?;

  // find the specified user
  let uid = conn.lookup_user(LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;

  conn.webpush_subscribe(uid, request)?;

  Ok(())
}

pub async fn dismiss(
  User(user): User,
  State(state): State<AppState>,
  Json(request): Json<WebPushDismissRequest>,
) -> Result<impl IntoResponse> {
  let InnerAppState { db, .. } = state.0.as_ref();

  let conn = db.get()?;

  // find the specified user
  let uid = conn.lookup_user(LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;

  conn.webpush_dismiss(uid, request)?;

  Ok(())
}
