use axum::{
  extract::{Query, State},
  response::IntoResponse,
  Json,
};
use serde::Deserialize;

use crate::{
  database::{
    query::{CreateQuery, CreateQueryRequest, ListQuery, ListQueryRequest, QueryConfig},
    user::{LookupUser, LookupUserRequest},
  },
  Error, Result,
};

use super::{
  state::{AppState, InnerAppState},
  User,
};

pub async fn create(
  User(user): User,
  State(state): State<AppState>,
  Json(config): Json<QueryConfig>,
) -> Result<impl IntoResponse> {
  let InnerAppState { db, .. } = state.0.as_ref();

  let mut conn = db.get()?;
  let tx = conn.transaction()?;

  // find the specified user
  let uid = tx.lookup_user(LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.auth.id,
  })?;

  // create query
  let qid = tx.create_query(CreateQueryRequest {
    uid,
    config: &config,
  })?;
  tx.commit()?;

  // note if create succeed
  log::debug!("created query [{}] for: [{}]", qid, uid);
  Ok(())
}

#[derive(Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum QueryType {
  PVP,
  Coop,
  Gears,
}

#[derive(Deserialize)]
pub struct ListRequest {
  pub qid: Option<i64>,
  pub qtype: Option<QueryType>,
}

pub async fn list(
  User(user): User,
  State(state): State<AppState>,
  Query(request): Query<ListRequest>,
) -> Result<impl IntoResponse> {
  let InnerAppState { db, .. } = state.0.as_ref();
  let ListRequest { qid, qtype } = request;

  let mut conn = db.get()?;
  let tx = conn.transaction()?;

  // find the specified user
  let uid = tx.lookup_user(LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.auth.id,
  })?;

  // if let Some(query) = request.query;
  let mut li = Vec::new();

  if qtype.is_none() || qtype.unwrap() == QueryType::PVP {
    li.append(&mut tx.list_query(ListQueryRequest { uid, qid })?);
  }

  let resp = serde_json::to_string(&li).map_err(|err| Error::InternalServerError(Box::new(err)))?;
  Ok(resp)
}
