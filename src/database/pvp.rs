use appendlist::AppendList;
use chrono::{DateTime, Datelike, Timelike, Utc};
use r2d2_sqlite::rusqlite::Connection;

use crate::{
  splatnet::{PvpMode, PvpRule},
  Error, Result,
};

use super::TimeZone;

fn fold_stage_mask(stages: &[u32]) -> u32 {
  stages.iter().fold(0u32, |a, b| a | (1 << (b - 1)))
}

#[derive(Debug)]
pub struct PvpQueryRecord {
  pub modes: u8,
  pub rules: u8,
  pub includes: u32,
  pub excludes: u32,
}

#[derive(Debug)]
pub struct CreatePvpQueryRequest<'a> {
  pub uid: i64,
  pub record: &'a PvpQueryRecord,
}

pub trait CreatePvpQuery {
  fn create_pvp_query(&self, request: CreatePvpQueryRequest) -> Result<i64>;
}

#[derive(Debug)]
pub struct LookupPvpRequest<'a> {
  pub start_time: DateTime<Utc>,
  pub rule: PvpRule,
  pub mode: PvpMode,
  pub stages: &'a [u32],
}

pub struct LookupPvpResponse {
  pub id: i64,
  pub uid: i64,
  pub agent: String,
}

pub trait LookupPvp {
  fn lookup_pvp(&self, request: LookupPvpRequest) -> Result<AppendList<LookupPvpResponse>>;
}

#[derive(Debug)]
pub struct ListPvpQueryRequest {
  pub uid: i64,
  pub qid: Option<i64>,
}

pub struct ListPvpQueryResponse {
  pub qid: i64,
  pub record: PvpQueryRecord,
  pub created_time: String,
}

pub trait ListPvpQuery {
  fn list_pvp_query(&self, request: ListPvpQueryRequest) -> Result<Vec<ListPvpQueryResponse>>;
}

#[derive(Debug)]
pub struct UpdatePvpQueryRequest<'a> {
  pub uid: i64,
  pub qid: i64,
  pub record: &'a PvpQueryRecord,
}

pub trait UpdatePvpQuery {
  fn update_pvp_query(&self, request: UpdatePvpQueryRequest) -> Result<()>;
}

#[derive(Debug)]
pub struct DeletePvpQueryRequest {
  pub uid: i64,
  pub qid: i64,
}

pub trait DeletePvpQuery {
  fn delete_pvp_query(&self, request: DeletePvpQueryRequest) -> Result<()>;
}

impl CreatePvpQuery for Connection {
  fn create_pvp_query(&self, request: CreatePvpQueryRequest) -> Result<i64> {
    let CreatePvpQueryRequest {
      uid,
      record:
        PvpQueryRecord {
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

impl LookupPvp for Connection {
  fn lookup_pvp(&self, request: LookupPvpRequest) -> Result<AppendList<LookupPvpResponse>> {
    let LookupPvpRequest {
      start_time,
      rule,
      mode,
      stages,
    } = request;
    let mode = mode as u8;
    let rule = rule as u8;
    let stages = fold_stage_mask(stages);
    let ts = start_time.timestamp();
    let t = TimeZone::Jst.convert(start_time);
    let a = t.weekday() as u32;
    let b = t.hour() / 2;
    let day_hrs = if a < 4 { "day_hrs_0" } else { "day_hrs_1" };
    let day_hrs_v = (1 << b) << 12 * (a % 4);
    // FIXME: add tests
    let sql = format!(
      "
      SELECT user_actions.id, uid_1, act_agent
      FROM (
        SELECT uid as uid_1
        FROM pvp_queries 
          INNER JOIN users ON uid = users.id
        WHERE 
          {day_hrs} & ?5 AND
          modes & ?1 AND 
          rules & ?2 AND 
          includes & ?3 AND 
          NOT (excludes & ?3)
      ) 
        INNER JOIN user_action_agents ON uid_1 == user_action_agents.uid
        INNER JOIN user_actions ON aid == user_action_agents.id
      WHERE act_active AND rx_pvp < ?4
      "
    );
    let mut stmt = self.prepare_cached(&sql)?;
    let iter = stmt.query_map((&mode, &rule, &stages, &ts, &day_hrs_v), |row| {
      Ok(LookupPvpResponse {
        id: row.get(0)?,
        uid: row.get(1)?,
        agent: row.get(2)?,
      })
    })?;
    let list = itertools::process_results(iter, |iter| iter.collect())?;
    Ok(list)
  }
}

impl ListPvpQuery for Connection {
  fn list_pvp_query(&self, request: ListPvpQueryRequest) -> Result<Vec<ListPvpQueryResponse>> {
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
      Ok(ListPvpQueryResponse {
        qid: row.get(0)?,
        record: PvpQueryRecord {
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

impl UpdatePvpQuery for Connection {
  fn update_pvp_query(&self, request: UpdatePvpQueryRequest) -> Result<()> {
    let UpdatePvpQueryRequest {
      uid,
      qid,
      record:
        PvpQueryRecord {
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

impl DeletePvpQuery for Connection {
  fn delete_pvp_query(&self, request: DeletePvpQueryRequest) -> Result<()> {
    let DeletePvpQueryRequest { uid, qid } = request;
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
        CreateQuery, CreateQueryRequest, PvpQueryConfig, QueryConfig, UpdateQuery,
        UpdateQueryRequest,
      },
      user::{CreateUser, CreateUserRequest, LookupUserId, LookupUserIdRequest},
      Database,
    },
    splatnet::PvpMode,
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
        language: None,
        time_zone: None,
        day_hrs: None,
      })
      .unwrap();
    assert!(ok);
    let uid = conn
      .lookup_user_id(LookupUserIdRequest {
        auth_agent,
        auth_uid,
      })
      .unwrap();

    let tx = conn.transaction().unwrap();
    let qid = tx
      .create_query(CreateQueryRequest {
        uid,
        config: &QueryConfig::Pvp {
          config: PvpQueryConfig {
            modes: vec![PvpMode::Open, PvpMode::X],
            rules: vec![PvpRule::Asari],
            includes: vec![1, 2],
            excludes: vec![4, 5],
          },
        },
      })
      .unwrap();
    tx.commit().unwrap();

    let li = conn
      .lookup_pvp(LookupPvpRequest {
        start_time: Utc::now(),
        rule: PvpRule::Asari,
        mode: PvpMode::X,
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
      .lookup_pvp(LookupPvpRequest {
        start_time: Utc::now(),
        rule: PvpRule::Asari,
        mode: PvpMode::X,
        stages: &[1, 3],
      })
      .unwrap();
    assert_eq!(li.len(), 1);
    let e = li.get(0).unwrap();
    assert_eq!(e.id, id);
    assert_eq!(e.uid, uid);
    assert_eq!(e.agent, act_agent);

    let li = conn
      .lookup_pvp(LookupPvpRequest {
        start_time: Utc::now(),
        rule: PvpRule::Hoko,
        mode: PvpMode::X,
        stages: &[1, 3],
      })
      .unwrap();
    // rule mismatch
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(LookupPvpRequest {
        start_time: Utc::now(),
        rule: PvpRule::Asari,
        mode: PvpMode::Challenge,
        stages: &[1, 3],
      })
      .unwrap();
    // mode mismatch
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(LookupPvpRequest {
        start_time: Utc::now(),
        rule: PvpRule::Asari,
        mode: PvpMode::X,
        stages: &[1, 2],
      })
      .unwrap();
    // all match
    assert_eq!(li.len(), 1);

    let li = conn
      .lookup_pvp(LookupPvpRequest {
        start_time: Utc::now(),
        rule: PvpRule::Asari,
        mode: PvpMode::X,
        stages: &[1, 4],
      })
      .unwrap();
    // match excl
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(LookupPvpRequest {
        start_time: Utc::now(),
        rule: PvpRule::Asari,
        mode: PvpMode::X,
        stages: &[16, 17, 18],
      })
      .unwrap();
    // all neutral
    assert_eq!(li.len(), 0);

    let tx = conn.transaction().unwrap();
    tx.update_query(UpdateQueryRequest {
      uid,
      qid,
      config: &QueryConfig::Pvp {
        config: PvpQueryConfig {
          modes: vec![PvpMode::Open, PvpMode::X],
          rules: vec![PvpRule::Asari],
          includes: vec![10],
          excludes: vec![1, 2],
        },
      },
    })
    .unwrap();
    tx.commit().unwrap();

    let li = conn
      .lookup_pvp(LookupPvpRequest {
        start_time: Utc::now(),
        rule: PvpRule::Asari,
        mode: PvpMode::X,
        stages: &[1, 2],
      })
      .unwrap();
    // updated
    assert_eq!(li.len(), 0);
  }
}
