use axum::{extract::State, response::IntoResponse, Json};

use crate::{
  action::webpush::{WebPushSubscribe, WebPushSubscribeRequest},
  api::{
    state::{AppState, InnerAppState},
    User,
  },
  database::user::{LookupUserId, LookupUserIdRequest},
  Result,
};

pub async fn subscribe(
  User(user): User,
  State(state): State<AppState>,
  Json(request): Json<WebPushSubscribeRequest>,
) -> Result<impl IntoResponse> {
  let InnerAppState { db, .. } = state.0.as_ref();

  let mut conn = db.get()?;

  // find the specified user
  let uid = conn.lookup_user_id(LookupUserIdRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;

  let tx = conn.transaction()?;
  let id = tx.webpush_subscribe(uid, request)?;
  tx.commit()?;

  Ok(id.to_string())
}
