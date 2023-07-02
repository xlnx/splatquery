mod config;

use std::{fs::File, io::BufReader, sync::Arc};

use splatquery::{
  action::infolog::InfoLogActionAgent,
  database::{
    pvp::{CreatePVPQuery, CreatePVPQueryRequest},
    user::{
      CreateUser, CreateUserRequest, LookupUser, LookupUserRequest, UpdateUserAction,
      UpdateUserActionRequest,
    },
    Database,
  },
  errors::BoxError,
  splatnet::SplatNet,
};

use crate::config::{Config, QueryConfig};

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
  let mut actions = config.agents.collect()?;
  actions.insert("infolog", Arc::new(InfoLogActionAgent()));
  log::debug!("agents = {:?}", actions);

  // prepare database agent
  let db = Database::new_in_memory()?;
  let mut conn = db.get()?;

  // prepare splatnet agent
  let splatnet = SplatNet::new(db.clone(), Arc::new(actions), config.splatnet);

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
    match query {
      QueryConfig::PVP {
        modes,
        rules,
        stages,
      } => {
        for rule in rules.iter() {
          let incl = stages
            .includes
            .iter()
            .map(|stage| splatnet.get_stage_id(&stage));
          let incl: Vec<_> = itertools::process_results(incl, |iter| iter.collect())?;
          let excl = stages
            .excludes
            .iter()
            .map(|stage| splatnet.get_stage_id(&stage));
          let excl: Vec<_> = itertools::process_results(excl, |iter| iter.collect())?;
          let qid = tx.create_pvp_query(&CreatePVPQueryRequest {
            uid,
            modes: &modes,
            rules: &[(*rule, &incl, &excl)],
          })?;
          log::debug!("new query qid=[{}]", qid);
        }
      }
    }
    tx.commit()?;
  }

  splatnet.watch().await;

  Ok(())
}
