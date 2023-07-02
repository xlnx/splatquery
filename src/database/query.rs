use std::collections::HashMap;

use appendlist::AppendList;
use rusqlite::Transaction;
use serde::{Deserialize, Serialize};

use crate::{
  database::pvp::{CreatePVPQueryRequest, Rule},
  errors::{Error, Result},
  splatnet::{PVPMode, SplatNet},
};

use super::pvp::CreatePVPQuery;

#[derive(Serialize, Deserialize)]
pub struct QueryPVPStagesConfig {
  pub includes: Vec<String>,
  #[serde(default)]
  pub excludes: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryPVPRuleStageConfig {
  Product {
    #[serde(default = "default_query_pvp_product_rules")]
    rules: Vec<Rule>,
    #[serde(flatten)]
    stages: QueryPVPStagesConfig,
  },
  Manual {
    #[serde(flatten)]
    rules: HashMap<Rule, QueryPVPStagesConfig>,
  },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum QueryConfig {
  PVP {
    #[serde(default = "default_query_pvp_modes")]
    modes: Vec<PVPMode>,
    stages: QueryPVPRuleStageConfig,
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
  pub splatnet: &'a SplatNet,
  pub query: &'a QueryConfig,
}

pub trait CreateQuery {
  fn create_query(&self, request: &CreateQueryRequest) -> Result<i64>;
}

impl<'a> CreateQuery for Transaction<'a> {
  fn create_query(&self, request: &CreateQueryRequest) -> Result<i64> {
    let CreateQueryRequest {
      uid,
      splatnet,
      query,
    } = *request;

    match query {
      QueryConfig::PVP {
        modes,
        stages: rules,
      } => {
        let parse_stage_list = |stages: &[String]| -> Result<Vec<_>> {
          let mut stages_ = vec![];
          for stage in stages.iter() {
            let stage = splatnet
              .get_stage_id(&stage)
              .map_err(|_| Error::InvalidParameter("stage", stage.clone()))?;
            stages_.push(stage);
          }
          Ok(stages_)
        };
        let li = AppendList::new();
        let parse_stage_config = |config: &QueryPVPStagesConfig| -> Result<_> {
          let includes = li.push(parse_stage_list(config.includes.as_slice())?);
          let excludes = li.push(parse_stage_list(config.excludes.as_slice())?);
          Ok((includes.as_slice(), excludes.as_slice()))
        };
        let rules = match rules {
          QueryPVPRuleStageConfig::Product { rules, stages } => {
            // parse rules
            let (includes, excludes) = parse_stage_config(stages)?;
            rules
              .iter()
              .map(|rule| (*rule, includes, excludes))
              .collect()
          }
          QueryPVPRuleStageConfig::Manual { rules: lookup } => {
            // parse rules
            let mut rules = vec![];
            for (rule, stages) in lookup.into_iter() {
              let (includes, excludes) = parse_stage_config(stages)?;
              rules.push((*rule, includes, excludes))
            }
            rules
          }
        };
        // do create pvp query
        let id = self.create_pvp_query(&CreatePVPQueryRequest {
          uid,
          modes: &modes,
          rules: rules.as_slice(),
        })?;
        // emit id
        Ok(id)
      }
    }
  }
}
