use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use backoff::ExponentialBackoffBuilder;
use chrono::DateTime;
use futures::{future::join_all, Future, FutureExt, TryFutureExt};
use rusqlite::Connection;

use crate::{
  database::{
    pvp::{LookupPVP, LookupPVPRequest},
    Database,
  },
  splatnet::{Message, PVPRule},
  Error, Result,
};

pub mod config;
pub mod infolog;
#[cfg(feature = "webpush")]
pub mod webpush;

pub type ActionAgentMap = HashMap<&'static str, Arc<dyn ActionAgent>>;

#[async_trait]
pub trait ActionAgent: std::fmt::Debug + Send + Sync {
  fn get_ext_info<'a>(
    &self,
    _conn: &'a Connection,
    _id: i64,
  ) -> Result<Option<Box<dyn erased_serde::Serialize>>> {
    Ok(None)
  }

  async fn emit(self: Arc<Self>, db: Database, id: i64, msg: Arc<Message>) -> Result<()>;
}

#[derive(Clone)]
pub struct ActionManager {
  database: Database,
  pub agents: Arc<ActionAgentMap>,
}

impl ActionManager {
  pub fn new(database: Database, agents: ActionAgentMap) -> Self {
    ActionManager {
      database,
      agents: Arc::new(agents),
    }
  }

  pub fn dispatch(&self, msg: Message) -> Result<impl Future<Output = ()>> {
    let conn = self.database.get()?;
    let actions = match &msg {
      Message::PVP(item) => {
        let start_time = DateTime::parse_from_rfc3339(&item.start_time)
          .map_err(|err| Error::InternalServerError(Box::new(err)))?;
        let rule = PVPRule::from_base64(&item.rule);
        conn.lookup_pvp(LookupPVPRequest {
          start_time: start_time.into(),
          rule,
          mode: item.mode,
          stages: &item.stages,
        })?
      }
    };
    let msg = Arc::new(msg);
    let mut tasks = vec![];
    let exp = ExponentialBackoffBuilder::new()
      .with_initial_interval(Duration::from_secs(5))
      .with_max_interval(Duration::from_secs(60 * 10))
      .with_max_elapsed_time(Some(Duration::from_secs(60 * 60)))
      .build();
    for e in actions.iter() {
      if let Some(agent) = self.agents.get(e.agent.as_str()) {
        tasks.push(backoff::future::retry(exp.clone(), {
          let db = self.database.clone();
          let id = e.id;
          let uid = e.uid;
          let msg = msg.clone();
          let agent = agent.clone();
          let act_agent = e.agent.clone();
          let mut attempt_idx = 0;
          move || {
            attempt_idx += 1;
            agent.clone().emit(db.clone(), id, msg.clone()).map_err({
              let agent = act_agent.clone();
              move |err| {
                log::warn!(
                  "emit {}#{} for uid[{}] (attempt#{}) failed: [{:?}]",
                  agent,
                  id,
                  uid,
                  attempt_idx,
                  err
                );
                backoff::Error::transient(err)
              }
            })
          }
        }));
      } else {
        log::error!("unknown action agent: [{}]", e.agent)
      }
    }
    Ok(join_all(tasks.into_iter()).map(|_| ()))
  }
}
