use derivative::Derivative;
use futures::{future::join_all, Future, FutureExt};
use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use std::{pin::Pin, sync::Arc, time::Duration};
use strum_macros::EnumIter;
use tokio::sync::RwLock;
use tokio_stream::{wrappers::IntervalStream, StreamExt};

use crate::{action::ActionManager, BoxError};

use self::spider::Spider;
pub use self::spider::{CoopSpiderItem, GearSpiderItem, PvpSpiderItem};

mod gear;
pub mod i18n;
mod iso8601;
mod schedules;
mod spider;

#[derive(
  Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize_enum_str, Deserialize_enum_str, EnumIter,
)]
#[serde(rename_all = "lowercase")]
pub enum PvpMode {
  Unknown = 0,
  Regular = 1,
  Challenge = 2,
  Open = 4,
  X = 8,
  Fest = 16,
  Event = 32,
}

#[derive(
  Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize_enum_str, Deserialize_enum_str, EnumIter,
)]
#[serde(rename_all = "lowercase")]
pub enum PvpRule {
  Unknown = 0,
  Regular = 1,
  Area = 2,
  Yagura = 4,
  Hoko = 8,
  Asari = 16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GearType {
  Unknown = 0,
  Head = 1,
  Clothing = 2,
  Shoes = 4,
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

#[derive(Debug)]
pub enum Message {
  Pvp(PvpSpiderItem),
}

pub struct SplatNetAgent {
  actions: ActionManager,
  gear_update_interval: Duration,
  schedules_update_interval: Duration,
  state: RwLock<Spider>,
}

impl SplatNetAgent {
  pub fn new(actions: ActionManager, config: SplatNetConfig) -> Arc<Self> {
    Arc::new(SplatNetAgent {
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

  pub async fn watch(self: Arc<Self>) -> Result<(), BoxError> {
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
    futures::join!(update_gear, update_schedules);
    Ok(())
  }

  fn handle_error(&self, err: BoxError) {
    log::warn!("{:?}", err);
  }

  async fn handle_gear_update(&self, items: Vec<GearSpiderItem>) -> Result<(), BoxError> {
    Ok(())
  }

  async fn handle_pvp_update(&self, items: Vec<PvpSpiderItem>) -> Result<(), BoxError> {
    let mut tasks = vec![];
    for item in items.into_iter() {
      tasks.push(self.actions.dispatch(Message::Pvp(item))?);
    }
    join_all(tasks).await;
    Ok(())
  }

  async fn handle_coop_update(&self, items: Vec<CoopSpiderItem>) -> Result<(), BoxError> {
    Ok(())
  }
}
