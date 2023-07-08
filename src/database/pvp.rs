use appendlist::AppendList;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

use crate::{splatnet::PVPMode, Error, Result};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize_enum_str, Deserialize_enum_str)]
#[serde(rename_all = "lowercase")]
pub enum Rule {
  TurfWar = 1,
  Area = 2,
  Yagura = 4,
  Hoko = 8,
  Asari = 16,
  Unknown = 255,
}

impl Rule {
  pub fn from_base64(s: &str) -> Self {
    match s {
      "VnNSdWxlLTA=" => Self::TurfWar,
      "VnNSdWxlLTE=" => Self::Area,
      "VnNSdWxlLTI=" => Self::Yagura,
      "VnNSdWxlLTM=" => Self::Hoko,
      "VnNSdWxlLTQ=" => Self::Asari,
      _ => Self::Unknown,
    }
  }
}

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
  pub rule: Rule,
  pub mode: PVPMode,
  pub stages: &'a [u32],
}

#[derive(Debug, PartialEq, Eq)]
pub struct LookupPVPResponse {
  pub uid: i64,
  pub qid: i64,
  pub act_agent: String,
  pub act_config: String,
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
      Err(Error::SqliteError(rusqlite::Error::QueryReturnedNoRows))
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
    let mut stmt = self.prepare_cached(
      "
      SELECT 
        pvp_queries.uid as uid, 
        pvp_queries.id as id, 
        act_agent, 
        act_config
      FROM pvp_queries 
        INNER JOIN user_actions 
          ON pvp_queries.uid = user_actions.uid
      WHERE modes & ?1 AND rules & ?2 AND includes & ?3 AND NOT (excludes & ?3)
      ",
    )?;
    let iter = stmt.query_map((&mode, &rule, &stages), |row| {
      Ok(LookupPVPResponse {
        uid: row.get(0)?,
        qid: row.get(1)?,
        act_agent: row.get(2)?,
        act_config: row.get(3)?,
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
      Err(Error::SqliteError(rusqlite::Error::QueryReturnedNoRows))
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
      Err(Error::SqliteError(rusqlite::Error::QueryReturnedNoRows))
    } else {
      Ok(())
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    database::{
      query::{
        CreateQuery, CreateQueryRequest, PVPQueryConfig, QueryConfig, UpdateQuery,
        UpdateQueryRequest,
      },
      user::{
        CreateUser, CreateUserRequest, LookupUser, LookupUserRequest, UpdateUserAction,
        UpdateUserActionRequest,
      },
      Database,
    },
    splatnet::PVPMode,
  };

  use super::*;

  #[test]
  fn test_rule_deserialize() {
    let rule = Rule::from_base64("VnNSdWxlLTI=");
    assert_eq!(rule.to_string(), String::from("yagura"));
  }

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
            rules: vec![Rule::Asari],
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
        rule: Rule::Asari,
        mode: PVPMode::X,
        stages: &[1, 3],
      })
      .unwrap();
    // no action
    assert_eq!(li.len(), 0);

    let act_agent = "mock_act_agent";
    let act_config = "mock_act_config";
    conn
      .update_user_action(UpdateUserActionRequest {
        uid,
        act_agent,
        act_config,
      })
      .unwrap();

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: Rule::Asari,
        mode: PVPMode::X,
        stages: &[1, 3],
      })
      .unwrap();
    assert_eq!(li.len(), 1);
    assert_eq!(
      *li.get(0).unwrap(),
      LookupPVPResponse {
        uid: uid,
        qid: qid,
        act_agent: String::from(act_agent),
        act_config: String::from(act_config)
      }
    );

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: Rule::Hoko,
        mode: PVPMode::X,
        stages: &[1, 3],
      })
      .unwrap();
    // rule mismatch
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: Rule::Asari,
        mode: PVPMode::Challenge,
        stages: &[1, 3],
      })
      .unwrap();
    // mode mismatch
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: Rule::Asari,
        mode: PVPMode::X,
        stages: &[1, 2],
      })
      .unwrap();
    // all match
    assert_eq!(li.len(), 1);

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: Rule::Asari,
        mode: PVPMode::X,
        stages: &[1, 4],
      })
      .unwrap();
    // match excl
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(LookupPVPRequest {
        start_time: Utc::now(),
        rule: Rule::Asari,
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
          rules: vec![Rule::Asari],
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
        rule: Rule::Asari,
        mode: PVPMode::X,
        stages: &[1, 2],
      })
      .unwrap();
    // updated
    assert_eq!(li.len(), 0);
  }
}
