use axum::{
  extract::{Path, State},
  response::IntoResponse,
};

use crate::{
  database::user::{LookupUser, LookupUserRequest, UpdateUserAction, UpdateUserActionRequest},
  errors::{Error, Result},
};

use super::{
  state::{AppState, InnerAppState},
  User,
};

pub async fn update(
  Path(agent_type): Path<String>,
  User(user): User,
  State(state): State<AppState>,
  config: String,
) -> Result<impl IntoResponse> {
  let InnerAppState { actions, db, .. } = state.0.as_ref();

  // find the specified agent
  let agent = actions
    .get(agent_type.as_str())
    .ok_or_else(|| Error::InvalidParameter("agent_type", agent_type.clone()))?;
  log::debug!(
    "incoming [{}] action update request: [{:?}]",
    agent_type,
    config
  );
  // verify config format
  let _ = agent.clone().new_action(&config)?;

  let mut conn = db.get()?;
  let tx = conn.transaction()?;
  let uid = tx.lookup_user(&LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.auth.id,
  })?;
  tx.update_user_action(&UpdateUserActionRequest {
    uid,
    act_agent: &agent_type,
    act_config: &config,
  })?;
  tx.commit()?;

  // note if update succeed
  log::debug!("updated [{:?}] action for: [{}]", agent_type, uid);
  Ok(())
}
