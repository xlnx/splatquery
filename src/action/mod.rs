use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use backoff::ExponentialBackoffBuilder;
use futures::{future::join_all, Future, FutureExt, TryFutureExt};
use r2d2_sqlite::rusqlite::Connection;

#[cfg(feature = "renderer")]
use crate::renderer::Renderer;
use crate::{
  database::{
    pvp::{LookupPvp, LookupPvpRequest},
    Database,
  },
  splatnet::Message,
  Result,
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

  async fn emit(
    self: Arc<Self>,
    ctx: Arc<ActionContext>,
    uid: i64,
    id: i64,
    msg: Arc<Message>,
  ) -> Result<()>;

  async fn test(self: Arc<Self>, _db: Database, _uid: i64, _id: i64) -> Result<()> {
    Ok(())
  }
}

pub struct ActionContext {
  pub database: Database,
  #[cfg(feature = "renderer")]
  pub renderer: Arc<Renderer>,
  #[cfg(feature = "renderer")]
  pub image_url: String,
}

#[derive(Clone)]
pub struct ActionManager {
  ctx: Arc<ActionContext>,
  pub agents: Arc<ActionAgentMap>,
}

impl ActionManager {
  pub fn new(ctx: ActionContext, agents: ActionAgentMap) -> Self {
    ActionManager {
      ctx: Arc::new(ctx),
      agents: Arc::new(agents),
    }
  }

  pub fn dispatch(&self, msg: Message) -> Result<impl Future<Output = ()>> {
    let conn = self.ctx.database.get()?;
    let (actions, rx, ts) = match &msg {
      Message::Pvp(item) => (
        conn.lookup_pvp(LookupPvpRequest {
          start_time: item.start_time,
          rule: item.rule,
          mode: item.mode,
          stages: &item.stages,
        })?,
        "rx_pvp",
        item.start_time,
      ),
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
        let task = backoff::future::retry(exp.clone(), {
          let ctx = self.ctx.clone();
          let id = e.id;
          let uid = e.uid;
          let msg = msg.clone();
          let agent = agent.clone();
          let act_agent = e.agent.clone();
          let mut attempt_idx = 0;
          move || {
            attempt_idx += 1;
            agent
              .clone()
              .emit(ctx.clone(), uid, id, msg.clone())
              .map_err({
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
        });
        let task = task.and_then({
          let uid = e.uid;
          let id = e.id;
          let ts = ts.timestamp();
          let db = self.ctx.database.clone();
          move |()| async move {
            let conn = db.get()?;
            let sql = format!(
              "
              UPDATE user_actions
              SET {rx} = max({rx}, ?3)
              WHERE uid = ?1 AND id = ?2
              ",
              rx = rx
            );
            conn.execute(&sql, (&uid, &id, &ts))?;
            Ok(())
          }
        });
        tasks.push(task);
      } else {
        log::error!("unknown action agent: [{}]", e.agent)
      }
    }
    Ok(join_all(tasks.into_iter()).map(|_| ()))
  }
}
