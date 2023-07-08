use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

use crate::{
  database::pvp::{CreatePVPQueryRequest, Rule},
  splatnet::PVPMode,
  Error, Result,
};

use super::pvp::{
  CreatePVPQuery, DeletePVPQuery, DeletePVPQueryRequest, ListPVPQuery, ListPVPQueryRequest,
  PVPQueryRecord, UpdatePVPQuery, UpdatePVPQueryRequest,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_enum_str, Deserialize_enum_str)]
#[serde(rename_all = "lowercase")]
pub enum QueryType {
  PVP,
  Coop,
  Gears,
}

#[derive(Serialize, Deserialize)]
pub struct PVPQueryConfig {
  #[serde(default = "default_query_pvp_modes")]
  pub modes: Vec<PVPMode>,
  #[serde(default = "default_query_pvp_product_rules")]
  pub rules: Vec<Rule>,
  pub includes: Vec<u32>,
  #[serde(default)]
  pub excludes: Vec<u32>,
}

impl From<&PVPQueryRecord> for PVPQueryConfig {
  fn from(value: &PVPQueryRecord) -> Self {
    let parse_stage_list = |stages: u32| {
      let mut stages_ = vec![];
      for i in 0..32 {
        if ((1u32 << i) & stages) != 0 {
          stages_.push(i + 1);
        }
      }
      stages_
    };
    let parse_modes_list = |modes: u8| {
      let mut modes_ = vec![];
      for mode in [
        PVPMode::TurfWar,
        PVPMode::Challenge,
        PVPMode::Open,
        PVPMode::X,
      ] {
        if ((mode as u8) & modes) != 0 {
          modes_.push(mode);
        }
      }
      modes_
    };
    let parse_rules_list = |rules: u8| {
      let mut modes_ = vec![];
      for rule in [
        Rule::TurfWar,
        Rule::Area,
        Rule::Yagura,
        Rule::Hoko,
        Rule::Asari,
      ] {
        if ((rule as u8) & rules) != 0 {
          modes_.push(rule);
        }
      }
      modes_
    };
    let modes = parse_modes_list(value.modes);
    let rules = parse_rules_list(value.rules);
    let includes = parse_stage_list(value.includes);
    let excludes = parse_stage_list(value.excludes);
    PVPQueryConfig {
      modes,
      rules,
      includes,
      excludes,
    }
  }
}

impl TryInto<PVPQueryRecord> for &PVPQueryConfig {
  type Error = Error;

  fn try_into(self) -> std::result::Result<PVPQueryRecord, Self::Error> {
    let parse_stage_list = |stages: &[u32]| -> Result<_> {
      let mut ret = 0u32;
      for id in stages {
        ret |= id
          .checked_sub(1)
          .and_then(|e| 1u32.checked_shl(e))
          .ok_or_else(|| Error::InvalidParameter("stageid", id.to_string()))?;
      }
      Ok(ret)
    };
    let modes = self.modes.iter().fold(0u8, |a, b| a | *b as u8);
    let rules = self.rules.iter().fold(0u8, |a, b| a | *b as u8);
    let includes = parse_stage_list(&self.includes)?;
    let excludes = parse_stage_list(&self.excludes)?;
    Ok(PVPQueryRecord {
      modes,
      rules,
      includes,
      excludes,
    })
  }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum QueryConfig {
  PVP {
    #[serde(flatten)]
    config: PVPQueryConfig,
  },
}

fn default_query_pvp_product_rules() -> Vec<Rule> {
  vec![Rule::Area, Rule::Yagura, Rule::Hoko, Rule::Asari]
}

fn default_query_pvp_modes() -> Vec<PVPMode> {
  vec![
    PVPMode::TurfWar,
    PVPMode::Challenge,
    PVPMode::Open,
    PVPMode::X,
  ]
}

pub struct CreateQueryRequest<'a> {
  pub uid: i64,
  pub config: &'a QueryConfig,
}

pub trait CreateQuery {
  fn create_query(&self, request: CreateQueryRequest) -> Result<i64>;
}

pub struct ListQueryRequest {
  pub uid: i64,
  pub qid: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct ListQueryResponse {
  pub qid: i64,
  pub config: QueryConfig,
  pub created_time: String,
}

pub trait ListQuery {
  fn list_query(&self, request: ListQueryRequest) -> Result<Vec<ListQueryResponse>>;
}

pub struct UpdateQueryRequest<'a> {
  pub uid: i64,
  pub qid: i64,
  pub config: &'a QueryConfig,
}

pub trait UpdateQuery {
  fn update_query(&self, request: UpdateQueryRequest) -> Result<()>;
}

pub struct DeleteQueryRequest {
  pub uid: i64,
  pub qid: i64,
  pub qtype: QueryType,
}

pub trait DeleteQuery {
  fn delete_query(&self, request: DeleteQueryRequest) -> Result<()>;
}

impl CreateQuery for Connection {
  fn create_query(&self, request: CreateQueryRequest) -> Result<i64> {
    let CreateQueryRequest { uid, config } = request;
    match config {
      QueryConfig::PVP { config } => {
        // do create pvp query
        let record = &config.try_into()?;
        let id = self.create_pvp_query(CreatePVPQueryRequest { uid, record })?;
        // emit id
        Ok(id)
      }
    }
  }
}

impl ListQuery for Connection {
  fn list_query(&self, request: ListQueryRequest) -> Result<Vec<ListQueryResponse>> {
    let ListQueryRequest { uid, qid } = request;
    let li = self.list_pvp_query(ListPVPQueryRequest { uid, qid })?;
    let iter = li.into_iter().map(|e| ListQueryResponse {
      qid: e.qid,
      config: QueryConfig::PVP {
        config: (&e.record).into(),
      },
      created_time: e.created_time,
    });
    let li = iter.collect();
    Ok(li)
  }
}

impl UpdateQuery for Connection {
  fn update_query(&self, request: UpdateQueryRequest) -> Result<()> {
    let UpdateQueryRequest { uid, qid, config } = request;
    match config {
      QueryConfig::PVP { config } => {
        let record = &config.try_into()?;
        self.update_pvp_query(UpdatePVPQueryRequest { uid, qid, record })?;
        Ok(())
      }
    }
  }
}

impl DeleteQuery for Connection {
  fn delete_query(&self, request: DeleteQueryRequest) -> Result<()> {
    let DeleteQueryRequest { uid, qid, qtype } = request;
    match qtype {
      QueryType::PVP => {
        self.delete_pvp_query(DeletePVPQueryRequest { uid, qid })?;
        Ok(())
      }
      _ => Err(Error::InvalidParameter("qtype", qtype.to_string())),
    }
  }
}
