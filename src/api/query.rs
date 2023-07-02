use axum::{extract::State, response::IntoResponse, Json};

use crate::{
  database::{
    query::{CreateQuery, CreateQueryRequest, QueryConfig},
    user::{LookupUser, LookupUserRequest},
  },
  errors::Result,
};

use super::{
  state::{AppState, InnerAppState},
  User,
};

pub async fn create(
  User(user): User,
  State(state): State<AppState>,
  Json(query): Json<QueryConfig>,
) -> Result<impl IntoResponse> {
  let InnerAppState { db, splatnet, .. } = state.0.as_ref();

  let mut conn = db.get()?;
  let tx = conn.transaction()?;

  // find the specified user
  let uid = tx.lookup_user(&LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.auth.id,
  })?;

  // create query
  let qid = tx.create_query(&CreateQueryRequest {
    uid,
    splatnet,
    query: &query,
  })?;
  tx.commit()?;

  // note if create succeed
  log::debug!("created query [{}] for: [{}]", qid, uid);
  Ok(())
}
