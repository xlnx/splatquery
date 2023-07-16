use crate::{
  database::{
    action::{DeleteAction, ListAction, ToggleAction},
    user::{LookupUserId, LookupUserIdRequest},
  },
  Error, Result,
};
use axum::{
  extract::{Path, Query, State},
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use super::{
  state::{AppState, InnerAppState},
  User,
};

#[cfg(feature = "webpush")]
pub mod webpush;

#[derive(Serialize)]
pub struct ListResponse {
  id: i64,
  agent: String,
  active: bool,
  ext_info: Option<Box<dyn erased_serde::Serialize>>,
}

pub async fn list(User(user): User, State(state): State<AppState>) -> Result<impl IntoResponse> {
  let InnerAppState { db, actions, .. } = state.0.as_ref();
  let conn = db.get()?;
  let uid = conn.lookup_user_id(LookupUserIdRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;
  let mut li = vec![];
  for e in conn.list_action(uid)? {
    if let Some(agent) = actions.agents.get(e.agent.as_str()) {
      match agent.get_ext_info(&conn, e.id) {
        Ok(ext_info) => li.push(ListResponse {
          id: e.id,
          agent: e.agent.clone(),
          active: e.active,
          ext_info,
        }),
        Err(err) => log::warn!(
          "get_ext_info [{}] for action [{}] error: [{:?}]",
          e.agent,
          e.id,
          err
        ),
      }
    }
  }
  let resp = serde_json::to_string(&li).map_err(|err| Error::InternalServerError(Box::new(err)))?;
  Ok(resp)
}

#[derive(Deserialize)]
pub struct DeleteActionRequest {
  pub id: i64,
}

pub async fn delete(
  User(user): User,
  State(state): State<AppState>,
  Query(request): Query<DeleteActionRequest>,
) -> Result<()> {
  let InnerAppState { db, .. } = state.0.as_ref();
  let conn = db.get()?;
  let uid = conn.lookup_user_id(LookupUserIdRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;
  conn.delete_action(uid, request.id)?;
  Ok(())
}

#[derive(Deserialize)]
pub struct ToggleActionRequest {
  pub active: bool,
}

pub async fn toggle(
  User(user): User,
  State(state): State<AppState>,
  Path(agent): Path<String>,
  Query(request): Query<ToggleActionRequest>,
) -> Result<impl IntoResponse> {
  let InnerAppState { db, .. } = state.0.as_ref();
  let conn = db.get()?;
  let uid = conn.lookup_user_id(LookupUserIdRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;
  conn.toggle_action(uid, &agent, request.active)?;
  Ok(())
}

#[derive(Deserialize)]
pub struct TestActionRequest {
  id: i64,
}

pub async fn test(
  User(user): User,
  State(state): State<AppState>,
  Path(agent): Path<String>,
  Query(request): Query<TestActionRequest>,
) -> Result<()> {
  let InnerAppState { db, actions, .. } = state.0.as_ref();
  let conn = db.get()?;
  let agent = actions
    .agents
    .get(agent.as_str())
    .ok_or_else(|| Error::InvalidParameter("test", agent))?;
  let uid = conn.lookup_user_id(LookupUserIdRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;
  agent.clone().test(db.clone(), uid, request.id).await?;
  Ok(())
}
