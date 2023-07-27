use backoff::ExponentialBackoffBuilder;
use chrono::{Duration, DurationRound, Local, Utc};
use derivative::Derivative;
use futures::{future::join_all, Future, FutureExt};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use std::{pin::Pin, sync::Arc};
use strum_macros::EnumIter;
use tokio::{
  sync::RwLock,
  time::{sleep_until, Instant},
};

use crate::{action::ActionManager, BoxError};

use self::spider::Spider;
pub use self::spider::{CoopSpiderItem, GearSpiderItem, PvpSpiderItem};

mod gear;
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

impl PvpMode {
  pub fn title(self, locale: &str) -> String {
    t!(format!("modes.{}.title", self).as_str(), locale = locale)
  }

  pub fn name(self, locale: &str) -> String {
    t!(format!("modes.{}.name", self).as_str(), locale = locale)
  }

  pub fn img_url(self) -> String {
    match self {
      Self::Challenge | Self::Open => "img/mode/bankara.svg".into(),
      _ => format!("img/mode/{}.svg", self),
    }
  }
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

impl PvpRule {
  pub fn name(self, locale: &str) -> String {
    let b64 = base64::encode(format!("VsRule-{}", (self as u32).ilog2()));
    t!(
      format!("splatnet.rules.{}.name", b64).as_str(),
      locale = locale
    )
  }

  pub fn img_url(self) -> String {
    format!("img/rule/{}.svg", self)
  }
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
      gear_update_interval: Duration::minutes(config.update_interval_mins.gears),
      schedules_update_interval: Duration::minutes(config.update_interval_mins.schedules),
      state: RwLock::new(Spider::new()),
    })
  }

  pub async fn watch(self: Arc<Self>) -> Result<(), BoxError> {
    let update_gears = self.clone().poll(Duration::hours(4), |this| {
      Box::pin(async move {
        match this.state.write().await.update_gear().await {
          Ok(gears) => {
            if gears.is_empty() {
              false
            } else {
              log::info!("gears += {}", gears.len());
              this
                .handle_gear_update(gears)
                .await
                .unwrap_or_else(|err| this.handle_error(err));
              true
            }
          }
          Err(err) => {
            log::warn!("update gears failed: [{:?}]", err);
            false
          }
        }
      })
    });
    let update_schedules = self.clone().poll(Duration::hours(2), |this| {
      Box::pin(async move {
        match this.state.write().await.update_schedules().await {
          Ok((pvp, coop)) => {
            if pvp.is_empty() {
              false
            } else {
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
                .await;
              true
            }
          }
          Err(err) => {
            log::warn!("update schedules failed: [{:?}]", err);
            false
          }
        }
      })
    });
    futures::join!(update_gears, update_schedules);
    Ok(())
  }

  async fn poll(
    self: Arc<Self>,
    rotation: Duration,
    update: impl Fn(Arc<Self>) -> Pin<Box<dyn Future<Output = bool>>>,
  ) {
    let mut tick = Instant::now();
    loop {
      sleep_until(tick).await;
      // content not updated
      if !update(self.clone()).await {
        let exp = ExponentialBackoffBuilder::new()
          .with_initial_interval(Duration::seconds(5).to_std().unwrap())
          .with_max_interval(Duration::minutes(30).to_std().unwrap())
          .build();
        tick = backoff::future::retry(exp, || async {
          log::info!("retrying update...");
          let tick = Instant::now();
          if update(self.clone()).await {
            // content updated
            Ok(tick)
          } else {
            Err(backoff::Error::transient(()))
          }
        })
        .await
        .unwrap_or_else(|()| {
          log::warn!("update failed.");
          Instant::now()
        });
      }
      // measure tasks elapsed time
      let elapsed = Duration::from_std(Instant::now() - tick).unwrap();
      let fire = Utc::now() - elapsed;
      let eps = Duration::seconds(5);
      let next_fire = fire.duration_trunc(rotation).unwrap() + rotation + eps;
      tick += (next_fire - fire).to_std().unwrap();
      log::info!(
        "scheduled next tick at [{}]",
        next_fire.with_timezone(&Local)
      );
    }
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
