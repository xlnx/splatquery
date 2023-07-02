use appendlist::AppendList;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Transaction};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

use crate::{splatnet::PVPMode, Result};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize_enum_str, Deserialize_enum_str)]
#[serde(rename_all = "lowercase")]
pub enum Rule {
  TurfWar = 0,
  Area = 1,
  Yagura = 2,
  Hoko = 3,
  Asari = 4,
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
pub struct CreatePVPQueryRequest<'a> {
  pub uid: i64,
  pub modes: &'a [PVPMode],
  pub rules: &'a [(Rule, &'a [u32], &'a [u32])],
}

pub trait CreatePVPQuery {
  fn create_pvp_query(&self, request: &CreatePVPQueryRequest) -> Result<i64>;
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
  fn lookup_pvp(&self, request: &LookupPVPRequest) -> Result<AppendList<LookupPVPResponse>>;
}

impl<'a> CreatePVPQuery for Transaction<'a> {
  fn create_pvp_query(&self, request: &CreatePVPQueryRequest) -> Result<i64> {
    // insert rule predicates
    let mut masks = [[0, 0]; 5];
    for (rule, incl, excl) in request.rules.iter() {
      masks[*rule as usize] = [*incl, *excl].map(fold_stage_mask);
    }
    let mut stmt = self.prepare_cached(
      "
      INSERT INTO pvp_queries 
      ( uid, 
        area_incl, area_excl, 
        yagura_incl, yagura_excl, 
        hoko_incl, hoko_excl, 
        asari_incl, asari_excl )
      VALUES 
      (
        ?1,
        ?2, ?3,
        ?4, ?5,
        ?6, ?7,
        ?8, ?9 )
      ",
    )?;
    stmt.execute((
      &request.uid,
      &masks[Rule::Area as usize][0],
      &masks[Rule::Area as usize][1],
      &masks[Rule::Yagura as usize][0],
      &masks[Rule::Yagura as usize][1],
      &masks[Rule::Hoko as usize][0],
      &masks[Rule::Hoko as usize][1],
      &masks[Rule::Asari as usize][0],
      &masks[Rule::Asari as usize][1],
    ))?;
    let qid = self.last_insert_rowid();
    // insert mode predicates
    let mut stmt = self.prepare_cached(
      "
      INSERT INTO pvp_query_modes ( mode, qid )
      VALUES ( ?1, ?2 )
      ",
    )?;
    for mode in request.modes.iter() {
      stmt.execute((&(*mode as u8), &qid))?;
    }
    Ok(qid)
  }
}

impl LookupPVP for Connection {
  fn lookup_pvp(&self, request: &LookupPVPRequest) -> Result<AppendList<LookupPVPResponse>> {
    let mode = request.mode as u8;
    let rule = request.rule.to_string();
    let stages = fold_stage_mask(request.stages);
    let sql = format!(
      "
      SELECT 
        pvp_queries.uid as uid, 
        pvp_queries.id as id, 
        act_agent, 
        act_config
      FROM pvp_queries 
        INNER JOIN pvp_query_modes 
          ON pvp_queries.id = qid
        INNER JOIN user_actions 
          ON pvp_queries.uid = user_actions.uid
      WHERE mode = ?1 AND {rule}_incl & ?2 AND NOT ({rule}_excl & ?2)
      ",
      rule = rule,
    );
    if let Ok(mut stmt) = self.prepare_cached(&sql) {
      let iter = stmt.query_map((&mode, &stages), |row| {
        Ok(LookupPVPResponse {
          uid: row.get(0)?,
          qid: row.get(1)?,
          act_agent: row.get(2)?,
          act_config: row.get(3)?,
        })
      })?;
      let list = itertools::process_results(iter, |iter| iter.collect())?;
      Ok(list)
    } else {
      // skipped due to unknown rule
      Ok(AppendList::new())
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    database::{
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
      .create_user(&CreateUserRequest {
        auth_agent,
        auth_uid,
        email: None,
        name: None,
        picture: None,
      })
      .unwrap();
    assert!(ok);
    let uid = conn
      .lookup_user(&LookupUserRequest {
        auth_agent,
        auth_uid,
      })
      .unwrap();

    let tx = conn.transaction().unwrap();
    let qid = tx
      .create_pvp_query(&CreatePVPQueryRequest {
        uid,
        modes: &[PVPMode::Open, PVPMode::X],
        rules: &[(Rule::Asari, &[1, 2], &[4, 5])],
      })
      .unwrap();
    tx.commit().unwrap();

    let li = conn
      .lookup_pvp(&LookupPVPRequest {
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
      .update_user_action(&UpdateUserActionRequest {
        uid,
        act_agent,
        act_config,
      })
      .unwrap();

    let li = conn
      .lookup_pvp(&LookupPVPRequest {
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
      .lookup_pvp(&LookupPVPRequest {
        start_time: Utc::now(),
        rule: Rule::Hoko,
        mode: PVPMode::X,
        stages: &[1, 3],
      })
      .unwrap();
    // rule mismatch
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(&LookupPVPRequest {
        start_time: Utc::now(),
        rule: Rule::Asari,
        mode: PVPMode::Challenge,
        stages: &[1, 3],
      })
      .unwrap();
    // mode mismatch
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(&LookupPVPRequest {
        start_time: Utc::now(),
        rule: Rule::Asari,
        mode: PVPMode::X,
        stages: &[1, 2],
      })
      .unwrap();
    // all match
    assert_eq!(li.len(), 1);

    let li = conn
      .lookup_pvp(&LookupPVPRequest {
        start_time: Utc::now(),
        rule: Rule::Asari,
        mode: PVPMode::X,
        stages: &[1, 4],
      })
      .unwrap();
    // match excl
    assert_eq!(li.len(), 0);

    let li = conn
      .lookup_pvp(&LookupPVPRequest {
        start_time: Utc::now(),
        rule: Rule::Asari,
        mode: PVPMode::X,
        stages: &[16, 17, 18],
      })
      .unwrap();
    // all neutral
    assert_eq!(li.len(), 0);
  }
}
