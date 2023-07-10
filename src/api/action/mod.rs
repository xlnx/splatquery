#[cfg(feature = "webpush")]
use crate::action::webpush::{WebPushList, WebPushListResponse};
use crate::{
  database::{
    action::{ListAction, ToggleAction, ToggleActionRequest},
    user::{LookupUser, LookupUserRequest},
  },
  Error, Result,
};
use axum::{
  extract::{Path, Query, State},
  response::IntoResponse,
};
use serde::Serialize;

use super::{
  state::{AppState, InnerAppState},
  User,
};

#[cfg(feature = "webpush")]
pub mod webpush;

#[derive(Serialize, Default)]
pub struct ListResponse {
  actions: Vec<String>,
  #[cfg(feature = "webpush")]
  webpush: Option<Vec<WebPushListResponse>>,
}

pub async fn list(User(user): User, State(state): State<AppState>) -> Result<impl IntoResponse> {
  let InnerAppState { db, .. } = state.0.as_ref();
  let conn = db.get()?;
  let uid = conn.lookup_user(LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;
  let mut resp = ListResponse::default();
  resp.actions = conn.list_action(uid)?;
  #[cfg(feature = "webpush")]
  {
    resp.webpush = Some(conn.webpush_list(uid)?);
  }
  let resp =
    serde_json::to_string(&resp).map_err(|err| Error::InternalServerError(Box::new(err)))?;
  Ok(resp)
}

pub async fn toggle(
  User(user): User,
  State(state): State<AppState>,
  Path(agent): Path<String>,
  Query(request): Query<ToggleActionRequest>,
) -> Result<impl IntoResponse> {
  let InnerAppState { db, .. } = state.0.as_ref();
  let conn = db.get()?;
  let uid = conn.lookup_user(LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;
  conn.toggle_action(uid, &agent, request)?;
  Ok(())
}
