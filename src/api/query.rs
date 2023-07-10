use axum::{
  extract::{Query, State},
  response::IntoResponse,
  Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
  database::{
    query::{
      CreateQuery, CreateQueryRequest, DeleteQuery, DeleteQueryRequest, ListQuery,
      ListQueryRequest, QueryConfig, QueryType, UpdateQuery, UpdateQueryRequest,
    },
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

  let conn = db.get()?;

  // find the specified user
  let uid = conn.lookup_user(LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;

  // create query
  let qid = conn.create_query(CreateQueryRequest {
    uid,
    config: &config,
  })?;

  // note if create succeed
  log::debug!("created query [{}] for: [{}]", qid, uid);
  let resp = serde_json::to_string(&json!({ "qid": qid }))
    .map_err(|err| Error::InternalServerError(Box::new(err)))?;
  Ok(resp)
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

  let conn = db.get()?;

  // find the specified user
  let uid = conn.lookup_user(LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;

  let mut li = Vec::new();

  if qtype.is_none() || qtype.unwrap() == QueryType::PVP {
    li.append(&mut conn.list_query(ListQueryRequest { uid, qid })?);
  }

  let resp = serde_json::to_string(&li).map_err(|err| Error::InternalServerError(Box::new(err)))?;
  Ok(resp)
}

#[derive(Deserialize)]
pub struct UpdateRequest {
  pub qid: i64,
}

pub async fn update(
  User(user): User,
  State(state): State<AppState>,
  Query(request): Query<UpdateRequest>,
  Json(config): Json<QueryConfig>,
) -> Result<()> {
  let InnerAppState { db, .. } = state.0.as_ref();
  let UpdateRequest { qid } = request;

  let conn = db.get()?;

  // find the specified user
  let uid = conn.lookup_user(LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;

  conn.update_query(UpdateQueryRequest {
    uid,
    qid,
    config: &config,
  })?;

  Ok(())
}

#[derive(Deserialize)]
pub struct DeleteRequest {
  pub qid: i64,
  pub qtype: QueryType,
}

pub async fn delete(
  User(user): User,
  State(state): State<AppState>,
  Query(request): Query<DeleteRequest>,
) -> Result<()> {
  let InnerAppState { db, .. } = state.0.as_ref();
  let DeleteRequest { qid, qtype } = request;

  let conn = db.get()?;

  // find the specified user
  let uid = conn.lookup_user(LookupUserRequest {
    auth_agent: &user.agent,
    auth_uid: &user.id,
  })?;

  conn.delete_query(DeleteQueryRequest { uid, qid, qtype })?;

  Ok(())
}
