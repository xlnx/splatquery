use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use splatquery::{
  action::config::ActionAgentsConfig,
  database::pvp::Rule,
  splatnet::{PVPMode, SplatNetConfig},
};

#[derive(Serialize, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub splatnet: SplatNetConfig,
  #[serde(default)]
  pub agents: ActionAgentsConfig,
  pub actions: HashMap<String, Value>,
  #[serde(default)]
  pub queries: Vec<QueryConfig>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum QueryConfig {
  PVP {
    #[serde(default = "default_pvp_query_modes")]
    modes: Vec<PVPMode>,
    #[serde(default = "default_pvp_query_rules")]
    rules: Vec<Rule>,
    stages: QueryPVPStagesConfig,
  },
}

fn default_pvp_query_modes() -> Vec<PVPMode> {
  vec![
    PVPMode::TurfWar,
    PVPMode::Challenge,
    PVPMode::Open,
    PVPMode::X,
  ]
}

fn default_pvp_query_rules() -> Vec<Rule> {
  vec![Rule::Area, Rule::Yagura, Rule::Hoko, Rule::Asari]
}

#[derive(Serialize, Deserialize)]
pub struct QueryPVPStagesConfig {
  #[serde(default)]
  pub includes: Vec<String>,
  #[serde(default)]
  pub excludes: Vec<String>,
}
