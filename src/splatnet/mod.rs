use chrono::DateTime;
use derivative::Derivative;
use futures::{future::join_all, Future, FutureExt};
use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use serde_json::json;
use std::{pin::Pin, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tokio_stream::{wrappers::IntervalStream, StreamExt};

use crate::{
  action::ActionAgentMap,
  database::{
    pvp::{LookupPVP, LookupPVPRequest, Rule},
    Database,
  },
  BoxError,
};

use self::spider::{CoopSpiderItem, GearSpiderItem, PVPSpiderItem, Spider};

mod gear;
mod schedules;
mod spider;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize_enum_str, Deserialize_enum_str)]
#[serde(rename_all = "lowercase")]
pub enum PVPMode {
  TurfWar = 1,
  Challenge = 2,
  Open = 4,
  X = 8,
  Unknown = 255,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GearType {
  Head = 0,
  Clothing = 1,
  Shoes = 2,
  Unknown = 255,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("message")]
  Message(String),
}

#[derive(Serialize, Deserialize, Default)]
pub struct SplatNetConfig {
  #[serde(default)]
  pub update_interval_mins: SplatNetUpdateIntervalConfig,
}

#[derive(Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct SplatNetUpdateIntervalConfig {
  #[serde(default)]
  #[derivative(Default(value = "120"))]
  pub gears: i64,
  #[serde(default)]
  #[derivative(Default(value = "30"))]
  pub schedules: i64,
}

pub struct SplatNet {
  database: Database,
  actions: Arc<ActionAgentMap>,
  gear_update_interval: Duration,
  schedules_update_interval: Duration,
  state: RwLock<Spider>,
}

impl SplatNet {
  pub fn new(
    database: Database,
    actions: Arc<ActionAgentMap>,
    config: SplatNetConfig,
  ) -> Arc<Self> {
    Arc::new(SplatNet {
      database,
      actions,
      gear_update_interval: chrono::Duration::minutes(config.update_interval_mins.gears)
        .to_std()
        .unwrap(),
      schedules_update_interval: chrono::Duration::minutes(config.update_interval_mins.schedules)
        .to_std()
        .unwrap(),
      state: RwLock::new(Spider::new()),
    })
  }

  pub fn watch(self: Arc<Self>) -> impl Future<Output = ()> {
    let update_gear = {
      let mut timer = IntervalStream::new(tokio::time::interval(self.gear_update_interval));
      let this = self.clone();
      async move {
        while let Some(_) = timer.next().await {
          match this.state.write().await.update_gear().await {
            Ok(gears) => {
              log::info!("gears += {}", gears.len());
              this
                .handle_gear_update(gears)
                .await
                .unwrap_or_else(|err| this.handle_error(err))
            }
            Err(err) => {
              log::warn!("update gears failed: [{:?}]", err);
            }
          }
        }
      }
    };
    let update_schedules = {
      let mut timer = IntervalStream::new(tokio::time::interval(self.schedules_update_interval));
      let this = self.clone();
      async move {
        while let Some(_) = timer.next().await {
          match this.state.write().await.update_schedules().await {
            Ok((pvp, coop)) => {
              log::info!("pvp += {}, coop += {}", pvp.len(), coop.len());
              let tasks: [Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send>>; 2] = [
                Box::pin(this.handle_pvp_update(pvp)),
                Box::pin(this.handle_coop_update(coop)),
              ];
              join_all(tasks)
                .map(|rets| {
                  for ret in rets.into_iter() {
                    ret.unwrap_or_else(|err| this.handle_error(err));
                  }
                })
                .await
            }
            Err(err) => {
              log::warn!("update schedules failed: [{:?}]", err);
            }
          }
        }
      }
    };
    let tasks: [Pin<Box<dyn Future<Output = ()> + Send>>; 2] =
      [Box::pin(update_gear), Box::pin(update_schedules)];
    join_all(tasks).map(|_| ())
  }

  fn handle_error(&self, err: BoxError) {
    log::warn!("{:?}", err);
  }

  async fn handle_gear_update(&self, items: Vec<GearSpiderItem>) -> Result<(), BoxError> {
    Ok(())
  }

  async fn handle_pvp_update(&self, items: Vec<PVPSpiderItem>) -> Result<(), BoxError> {
    for item in items.into_iter() {
      if let Err(err) = self.handle_pvp_update_1(item).await {
        self.handle_error(err);
      }
    }
    Ok(())
  }

  async fn handle_pvp_update_1(&self, item: PVPSpiderItem) -> Result<(), BoxError> {
    let conn = self.database.get()?;
    let start_time = DateTime::parse_from_rfc3339(&item.start_time)?;
    let rule = Rule::from_base64(&item.rule);
    let actions = conn.lookup_pvp(LookupPVPRequest {
      start_time: start_time.into(),
      rule,
      mode: item.mode,
      stages: &item.stages,
    })?;
    if actions.len() > 0 {
      let item = Arc::new(json!({
        "type": "pvp",
        "start_time": item.start_time,
        "end_time": item.end_time,
        "rule": rule.to_string(),
        "stages": item.stages,
        "mode": item.mode.to_string(),
      }));
      for action in actions.into_iter() {
        let agent = self.actions.get(action.act_agent.as_str()).ok_or_else(|| {
          Error::Message(format!(
            "action agent [{}] is not registered",
            action.act_agent
          ))
        })?;
        let action = agent.clone().new_action(&action.act_config)?;
        // TODO: behaviour
        let item = item.clone();
        tokio::spawn(async move {
          action
            .emit(&item)
            .await
            .unwrap_or_else(|err| log::warn!("trigger action failed: [{:?}]", err));
        });
      }
    }
    Ok(())
  }

  async fn handle_coop_update(&self, items: Vec<CoopSpiderItem>) -> Result<(), BoxError> {
    Ok(())
  }
}
