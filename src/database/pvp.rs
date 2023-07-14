use appendlist::AppendList;
use chrono::{DateTime, Utc};
use r2d2_sqlite::rusqlite::Connection;

use crate::{
  splatnet::{PVPMode, PVPRule},
  Error, Result,
};

fn fold_stage_mask(stages: &[u32]) -> u32 {
  stages.iter().fold(0u32, |a, b| a | (1 << (b - 1)))
}

#[derive(Debug)]
pub struct PVPQueryRecord {
  pub modes: u8,
  pub rules: u8,
  pub includes: u32,
  pub excludes: u32,
}

#[derive(Debug)]
pub struct CreatePVPQueryRequest<'a> {
  pub uid: i64,
  pub record: &'a PVPQueryRecord,
}

pub trait CreatePVPQuery {
  fn create_pvp_query(&self, request: CreatePVPQueryRequest) -> Result<i64>;
}

#[derive(Debug)]
pub struct LookupPVPRequest<'a> {
  pub start_time: DateTime<Utc>,
  pub rule: PVPRule,
  pub mode: PVPMode,
  pub stages: &'a [u32],
}

pub struct LookupPVPResponse {
  pub id: i64,
  pub uid: i64,
  pub agent: String,
}

pub trait LookupPVP {
  fn lookup_pvp(&self, request: LookupPVPRequest) -> Result<AppendList<LookupPVPResponse>>;
}

#[derive(Debug)]
pub struct ListPVPQueryRequest {
  pub uid: i64,
  pub qid: Option<i64>,
}

pub struct ListPVPQueryResponse {
  pub qid: i64,
  pub record: PVPQueryRecord,
  pub created_time: String,
}

pub trait ListPVPQuery {
  fn list_pvp_query(&self, request: ListPVPQueryRequest) -> Result<Vec<ListPVPQueryResponse>>;
}

#[derive(Debug)]
pub struct UpdatePVPQueryRequest<'a> {
  pub uid: i64,
  pub qid: i64,
  pub record: &'a PVPQueryRecord,
}

pub trait UpdatePVPQuery {
  fn update_pvp_query(&self, request: UpdatePVPQueryRequest) -> Result<()>;
}

#[derive(Debug)]
pub struct DeletePVPQueryRequest {
  pub uid: i64,
  pub qid: i64,
}

pub trait DeletePVPQuery {
  fn delete_pvp_query(&self, request: DeletePVPQueryRequest) -> Result<()>;
}

impl CreatePVPQuery for Connection {
  fn create_pvp_query(&self, request: CreatePVPQueryRequest) -> Result<i64> {
    let CreatePVPQueryRequest {
      uid,
      record:
        PVPQueryRecord {
          modes,
          rules,
          includes,
          excludes,
        },
    } = request;
    let mut stmt = self.prepare_cached(
      "
      INSERT INTO pvp_queries ( uid, modes, rules, includes, excludes )
      VALUES ( ?1, ?2, ?3, ?4, ?5 )
      ",
    )?;
    let n = stmt.execute((&uid, &modes, &rules, &includes, &excludes))?;
    if n != 1 {
      Err(Error::SqliteError(
        r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows,
      ))
    } else {
      Ok(self.last_insert_rowid())
    }
  }
}

impl LookupPVP for Connection {
  fn lookup_pvp(&self, request: LookupPVPRequest) -> Result<AppendList<LookupPVPResponse>> {
    let mode = request.mode as u8;
    let rule = request.rule as u8;
    let stages = fold_stage_mask(request.stages);
    let start_time = request.start_time.timestamp();
    let mut stmt = self.prepare_cached(
      "
      SELECT user_actions.id, uid_1, act_agent
      FROM (
        SELECT uid as uid_1
        FROM pvp_queries 
        WHERE modes & ?1 AND rules & ?2 AND includes & ?3 AND NOT (excludes & ?3)
      ) 
        INNER JOIN user_action_agents ON uid_1 == user_action_agents.uid
        INNER JOIN user_actions ON aid == user_action_agents.id
      WHERE act_active AND rx_pvp < ?4
      ",
    )?;
    let iter = stmt.query_map((&mode, &rule, &stages, &start_time), |row| {
      Ok(LookupPVPResponse {
        id: row.get(0)?,
        uid: row.get(1)?,
        agent: row.get(2)?,
      })
    })?;
    let list = itertools::process_results(iter, |iter| iter.collect())?;
    Ok(list)
  }
}

impl ListPVPQuery for Connection {
  fn list_pvp_query(&self, request: ListPVPQueryRequest) -> Result<Vec<ListPVPQueryResponse>> {
    let mut sql: String = "
      SELECT id, modes, rules, includes, excludes, created_time
      FROM pvp_queries
      WHERE uid = ?1
      "
    .into();
    if request.qid.is_some() {
      sql += " AND id = ?2";
    } else {
      sql += " AND (1 OR ?2)";
    }
    let mut stmt = self.prepare_cached(&sql)?;
    let iter = stmt.query_map((&request.uid, &request.qid), |row| {
      Ok(ListPVPQueryResponse {
        qid: row.get(0)?,
        record: PVPQueryRecord {
          modes: row.get(1)?,
          rules: row.get(2)?,
          includes: row.get(3)?,
          excludes: row.get(4)?,
        },
        created_time: row.get(5)?,
      })
    })?;
    let li = itertools::process_results(iter, |iter| iter.collect())?;
    Ok(li)
  }
}

impl UpdatePVPQuery for Connection {
  fn update_pvp_query(&self, request: UpdatePVPQueryRequest) -> Result<()> {
    let UpdatePVPQueryRequest {
      uid,
      qid,
      record:
        PVPQueryRecord {
          modes,
          rules,
          includes,
          excludes,
        },
    } = request;
    let mut stmt = self.prepare_cached(
      "
      UPDATE pvp_queries
      SET modes = ?3, rules = ?4, includes = ?5, excludes = ?6
      WHERE uid = ?1 AND id = ?2
      ",
    )?;
    let n = stmt.execute((&uid, &qid, &modes, &rules, &includes, &excludes))?;
    if n != 1 {
      Err(Error::SqliteError(
        r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows,
      ))
    } else {
      Ok(())
    }
  }
}

impl DeletePVPQuery for Connection {
  fn delete_pvp_query(&self, request: DeletePVPQueryRequest) -> Result<()> {
    let DeletePVPQueryRequest { uid, qid } = request;
    let mut stmt = self.prepare_cached(
      "
      DELETE FROM pvp_queries
      WHERE uid = ?1 AND id = ?2",
    )?;
    let n = stmt.execute((&uid, &qid))?;
    if n != 1 {
      Err(Error::SqliteError(
        r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows,
      ))
    } else {
      Ok(())
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    database::{
      action::CreateAction,
      query::{
        CreateQuery, CreateQueryRequest, PVPQueryConfig, QueryConfig, UpdateQuery,
        UpdateQueryRequest,
      },
      user::{CreateUser, CreateUserRequest, LookupUser, LookupUserRequest},
      Database,
    },
    splatnet::PVPMode,
  };

  use super::*;

  #[test]
  fn test_lookup_simple() {
    let db = Database::new_in_memory().unwrap();
    let mut conn = db.get().unwrap();

    let auth_agent = "mock_auth_agent";
    let auth_uid = "mock_auth_uid";
    let ok = conn
      .create_user(CreateUserRequest {
        auth_agent,
        auth_uid,
        email: None,
        name: None,
        picture: None,
      })
      .unwrap();
    assert!(ok);
    let uid = conn
      .lookup_user(LookupUserRequest {
        auth_agent,
        auth_uid,
      })
      .unwrap();

    let tx = conn.transaction().unwrap();
    let qid = tx
      .create_query(CreateQueryRequest {
        uid,
        config: &QueryConfig::PVP {
          config: PVPQueryConfig {
            modes: vec![PVPMode::Open, PVPMode::X],
            rules: vec![PVPRule::Asari],
            includes: vec![1, 2],
            excludes: vec![4, 5],
          },
        },
      })
      .unwrap();
    tx.commit().unwrap();

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: PVPRule::Asari,
        mode: PVPMode::X,
        stages: &[1, 3],
      })
      .unwrap();
    // no action
    assert_eq!(li.len(), 0);

    let act_agent = "mock_act_agent";
    let tx = conn.transaction().unwrap();
    let id = tx.create_action(uid, act_agent).unwrap();
    tx.commit().unwrap();

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: PVPRule::Asari,
        mode: PVPMode::X,
        stages: &[1, 3],
      })
      .unwrap();
    assert_eq!(li.len(), 1);
    let e = li.get(0).unwrap();
    assert_eq!(e.id, id);
    assert_eq!(e.uid, uid);
    assert_eq!(e.agent, act_agent);

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: PVPRule::Hoko,
        mode: PVPMode::X,
        stages: &[1, 3],
      })
      .unwrap();
    // rule mismatch
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: PVPRule::Asari,
        mode: PVPMode::Challenge,
        stages: &[1, 3],
      })
      .unwrap();
    // mode mismatch
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: PVPRule::Asari,
        mode: PVPMode::X,
        stages: &[1, 2],
      })
      .unwrap();
    // all match
    assert_eq!(li.len(), 1);

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: PVPRule::Asari,
        mode: PVPMode::X,
        stages: &[1, 4],
      })
      .unwrap();
    // match excl
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: PVPRule::Asari,
        mode: PVPMode::X,
        stages: &[16, 17, 18],
      })
      .unwrap();
    // all neutral
    assert_eq!(li.len(), 0);

    let tx = conn.transaction().unwrap();
    tx.update_query(UpdateQueryRequest {
      uid,
      qid,
      config: &QueryConfig::PVP {
        config: PVPQueryConfig {
          modes: vec![PVPMode::Open, PVPMode::X],
          rules: vec![PVPRule::Asari],
          includes: vec![10],
          excludes: vec![1, 2],
        },
      },
    })
    .unwrap();
    tx.commit().unwrap();

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: PVPRule::Asari,
        mode: PVPMode::X,
        stages: &[1, 2],
      })
      .unwrap();
    // updated
    assert_eq!(li.len(), 0);
  }
}
