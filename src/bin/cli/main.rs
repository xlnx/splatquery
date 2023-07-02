use std::{collections::HashMap, fs::File, io::BufReader, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use splatquery::{
  action::{config::ActionAgentsConfig, infolog::InfoLogActionAgent},
  database::{
    query::{CreateQuery, CreateQueryRequest, QueryConfig},
    user::{
      CreateUser, CreateUserRequest, LookupUser, LookupUserRequest, UpdateUserAction,
      UpdateUserActionRequest,
    },
    Database,
  },
  errors::BoxError,
  splatnet::{SplatNet, SplatNetConfig},
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

#[tokio::main]
async fn main() -> Result<(), BoxError> {
  std::env::set_var(
    "RUST_LOG",
    "info,
    cli=debug,
    r2d2=off,
    splatquery=info,
    splatquery::action::infolog=info,
    splatquery::action::webpush=debug",
  );
  env_logger::init();

  // read config
  let path = std::env::args().skip(1).next().unwrap();
  let file = File::open(path)?;
  let reader = BufReader::new(file);
  let config: Config = serde_json::from_reader(reader)?;

  // prepare action agents
  let mut agents = config.agents.collect()?;
  agents.insert("infolog", Arc::new(InfoLogActionAgent()));
  log::debug!("agents = {:?}", agents);

  // prepare database agent
  let db = Database::new_in_memory()?;
  let mut conn = db.get()?;

  // prepare splatnet agent
  let splatnet = SplatNet::new(db.clone(), Arc::new(agents), config.splatnet);

  // prepare user
  let auth_agent = "";
  let auth_uid = "";
  let ok = conn.create_user(&CreateUserRequest {
    auth_agent,
    auth_uid,
    name: None,
    email: None,
    picture: None,
  })?;
  assert!(ok);
  let uid = conn.lookup_user(&LookupUserRequest {
    auth_agent,
    auth_uid,
  })?;

  // prepare use actions
  for (agent, config) in config.actions.iter() {
    conn.update_user_action(&UpdateUserActionRequest {
      uid,
      act_agent: &agent,
      act_config: &config.to_string(),
    })?;
  }

  // prepare user queries
  if config.queries.is_empty() {
    log::warn!("at least one query should be specified");
  }
  for query in config.queries.into_iter() {
    let splatnet = splatnet.clone();
    let tx = conn.transaction()?;
    tx.create_query(&CreateQueryRequest {
      uid,
      splatnet: splatnet.as_ref(),
      query: &query,
    })?;
    tx.commit()?;
  }

  splatnet.watch().await;

  Ok(())
}
