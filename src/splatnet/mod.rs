use derivative::Derivative;
use futures::{future::join_all, Future, FutureExt};
use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use std::{pin::Pin, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tokio_stream::{wrappers::IntervalStream, StreamExt};

use crate::{action::ActionManager, BoxError};

use self::spider::Spider;
pub use self::spider::{CoopSpiderItem, GearSpiderItem, PVPSpiderItem};

mod gear;
pub mod i18n;
mod schedules;
mod spider;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize_enum_str, Deserialize_enum_str)]
#[serde(rename_all = "lowercase")]
pub enum PVPMode {
  TurfWar = 1,
  Challenge = 2,
  Open = 4,
  X = 8,
  Fest = 16,
  Unknown = 255,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize_enum_str, Deserialize_enum_str)]
#[serde(rename_all = "lowercase")]
pub enum PVPRule {
  TurfWar = 1,
  Area = 2,
  Yagura = 4,
  Hoko = 8,
  Asari = 16,
  Unknown = 255,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize_enum_str, Deserialize_enum_str)]
#[serde(rename_all = "lowercase")]
pub enum Region {
  US,
  EU,
  CN,
  JP,
}

impl PVPRule {
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

#[derive(Debug)]
pub enum Message {
  PVP(PVPSpiderItem),
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

  async fn handle_pvp_update(&self, items: Vec<PVPSpiderItem>) -> Result<(), BoxError> {
    let mut tasks = vec![];
    for item in items.into_iter() {
      tasks.push(self.actions.dispatch(Message::PVP(item))?);
    }
    join_all(tasks).await;
    Ok(())
  }

  async fn handle_coop_update(&self, items: Vec<CoopSpiderItem>) -> Result<(), BoxError> {
    Ok(())
  }
}
