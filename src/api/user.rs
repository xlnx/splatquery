use axum::{extract::State, response::IntoResponse, Json};

use crate::{
  database::user::{
    ListUserSettings, LookupUserId, LookupUserIdRequest, UpdateUserSettings, UserSettings,
  },
  Error, Result,
};

use super::{
  state::{AppState, InnerAppState},
  User,
};

pub async fn list(User(user): User, State(state): State<AppState>) -> Result<impl IntoResponse> {
  let InnerAppState { db, .. } = state.0.as_ref();

  let conn = db.get()?;

  let uid = conn.lookup_user_id(LookupUserIdRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;

  let settings = conn.list_user_settings(uid)?;

  let resp =
    serde_json::to_string(&settings).map_err(|err| Error::InternalServerError(Box::new(err)))?;
  Ok(resp)
}

pub async fn update(
  User(user): User,
  State(state): State<AppState>,
  Json(settings): Json<UserSettings>,
) -> Result<impl IntoResponse> {
  let InnerAppState { db, .. } = state.0.as_ref();

  let conn = db.get()?;

  let uid = conn.lookup_user_id(LookupUserIdRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;

  conn.update_user_settings(uid, &settings)?;

  Ok(())
}
