use chrono::{DateTime, Local, Utc};

use crate::BoxError;

use super::{
  gear,
  schedules::{self, RawCoopNormalSchedule},
  GearType, PvpMode, PvpRule,
};

#[derive(Debug)]
pub struct GearSpiderItem {
  pub sale_end_time: DateTime<Utc>,
  pub id: String,
  pub splatoon3ink_id: String,
  pub gear_type: GearType,
  pub brand: String,
  pub price: i32,
  pub primary_gear_power: String,
  pub additional_gear_powers: i32,
}

#[derive(Debug, Clone)]
pub struct PvpSpiderItem {
  pub start_time: DateTime<Utc>,
  pub end_time: DateTime<Utc>,
  pub rule: PvpRule,
  pub stages: Vec<u32>,
  pub mode: PvpMode,
}

#[derive(Debug)]
pub struct CoopSpiderItem {
  pub start_time: DateTime<Utc>,
  pub end_time: DateTime<Utc>,
  stage: String,
  weapons: String,
  king_salmonid_guess: String,
}

pub struct Spider {
  gear_pickup_brand: DateTime<Utc>,
  gear_limited: DateTime<Utc>,
  pvp_regular: DateTime<Utc>,
  pvp_bankara: DateTime<Utc>,
  pvp_x_match: DateTime<Utc>,
  pvp_event: DateTime<Utc>,
  pvp_fest: DateTime<Utc>,
  coop_normal: DateTime<Utc>,
  coop_big_run: DateTime<Utc>,
  coop_team_contest: DateTime<Utc>,
}

impl Spider {
  pub fn new() -> Self {
    // TODO: support suspend state
    Spider {
      gear_pickup_brand: DateTime::<Utc>::MIN_UTC,
      gear_limited: DateTime::<Utc>::MIN_UTC,
      pvp_regular: DateTime::<Utc>::MIN_UTC,
      pvp_bankara: DateTime::<Utc>::MIN_UTC,
      pvp_x_match: DateTime::<Utc>::MIN_UTC,
      pvp_event: DateTime::<Utc>::MIN_UTC,
      pvp_fest: DateTime::<Utc>::MIN_UTC,
      coop_normal: DateTime::<Utc>::MIN_UTC,
      coop_big_run: DateTime::<Utc>::MIN_UTC,
      coop_team_contest: DateTime::<Utc>::MIN_UTC,
    }
  }

  pub async fn update_gear(&mut self) -> Result<Vec<GearSpiderItem>, BoxError> {
    let url = "https://splatoon3.ink/data/gear.json";
    let response = reqwest::get(url).await?;
    log::debug!("GET [{}] -> {}", url, response.status().as_u16());
    let json: gear::RawGearResponse = response.json().await?;
    self.do_update_gear(json).await
  }

  pub async fn update_schedules(
    &mut self,
  ) -> Result<(Vec<PvpSpiderItem>, Vec<CoopSpiderItem>), BoxError> {
    let url = "https://splatoon3.ink/data/schedules.json";
    let response = reqwest::get(url).await?;
    log::debug!("GET [{}] -> {}", url, response.status().as_u16());
    let json: schedules::RawSchedulesResponse = response.json().await?;
    self.do_update_schedules(json).await
  }

  async fn do_update_gear(
    &mut self,
    response: gear::RawGearResponse,
  ) -> Result<Vec<GearSpiderItem>, BoxError> {
    let gear::RawGesoTown {
      pickup_brand,
      limited_gears,
    } = response.data.gesotown;

    let mut gears = vec![];
    let mut collect = |e: gear::RawGear| {
      gears.push(GearSpiderItem {
        sale_end_time: e.sale_end_time,
        id: e.id,
        splatoon3ink_id: e.gear.splatoon3ink_id,
        gear_type: match e.gear.typename.as_str() {
          "HeadGear" => GearType::Head,
          "ClothingGear" => GearType::Clothing,
          "ShoesGear" => GearType::Shoes,
          _ => GearType::Unknown,
        },
        brand: e.gear.brand.id,
        price: e.price,
        primary_gear_power: e.gear.primary_gear_power.splatoon3ink_id,
        additional_gear_powers: e.gear.additional_gear_powers.len() as i32,
      });
    };

    let mut t = pickup_brand.sale_end_time;
    if t > self.gear_pickup_brand {
      // found new pickup brand
      std::mem::swap(&mut self.gear_pickup_brand, &mut t);
      log::debug!(
        "cursor.gear_pickup_brand [{}] -> [{}]",
        t.with_timezone(&Local),
        self.gear_pickup_brand.with_timezone(&Local)
      );
      for g in pickup_brand.brand_gears.into_iter() {
        collect(g);
      }
    }

    if let Some(g) = limited_gears.last() {
      let mut t = g.sale_end_time;
      if t > self.gear_limited {
        // found new limited gears
        std::mem::swap(&mut self.gear_limited, &mut t);
        log::debug!(
          "cursor.gear_limited [{}] -> [{}]",
          t.with_timezone(&Local),
          self.gear_limited.with_timezone(&Local)
        );
        for g in limited_gears.into_iter() {
          if g.sale_end_time > t {
            collect(g);
          }
        }
      }
    }

    Ok(gears)
  }

  async fn do_update_schedules(
    &mut self,
    response: schedules::RawSchedulesResponse,
  ) -> Result<(Vec<PvpSpiderItem>, Vec<CoopSpiderItem>), BoxError> {
    let schedules::RawSchedulesData {
      regular_schedules,
      bankara_schedules,
      x_schedules,
      fest_schedules,
      coop_grouping_schedule,
      ..
    } = response.data;

    let mut pvp = vec![];
    let mut collect_pvp = |mode: PvpMode,
                           time_period: schedules::RawTimePeriod,
                           setting: schedules::RawPvpMatchSetting| {
      let rule = match setting.pvp_rule.id.as_str() {
        "VnNSdWxlLTA=" => PvpRule::Regular,
        "VnNSdWxlLTE=" => PvpRule::Area,
        "VnNSdWxlLTI=" => PvpRule::Yagura,
        "VnNSdWxlLTM=" => PvpRule::Hoko,
        "VnNSdWxlLTQ=" => PvpRule::Asari,
        _ => PvpRule::Unknown,
      };
      pvp.push(PvpSpiderItem {
        start_time: time_period.start_time,
        end_time: time_period.end_time,
        rule,
        stages: setting
          .pvp_stages
          .into_iter()
          .map(|e| e.pvp_stage_id)
          .collect(),
        mode,
      });
    };

    let mut coop = vec![];
    let mut collect_coop = |schedule: RawCoopNormalSchedule| {
      coop.push(CoopSpiderItem {
        start_time: schedule.time_period.start_time,
        end_time: schedule.time_period.end_time,
        stage: schedule.setting.coop_stage.id,
        weapons: schedule
          .setting
          .weapons
          .into_iter()
          .map(|e| e.splatoon3ink_id)
          .collect(),
        king_salmonid_guess: schedule.splatoon3ink_king_salmonid_guess,
      });
    };

    if let Some(s) = regular_schedules.nodes.last() {
      let mut t = s.time_period.start_time;
      if t > self.pvp_regular {
        // find new turf-war schedule
        std::mem::swap(&mut self.pvp_regular, &mut t);
        log::debug!(
          "cursor.pvp_regular [{}] -> [{}]",
          t.with_timezone(&Local),
          self.pvp_regular.with_timezone(&Local)
        );
        for s in regular_schedules.nodes.into_iter() {
          if s.time_period.start_time > t {
            if let Some(setting) = s.match_setting.regular_match_setting {
              collect_pvp(PvpMode::Regular, s.time_period, setting);
            }
          }
        }
      }
    }

    if let Some(s) = bankara_schedules.nodes.last() {
      let mut t = s.time_period.start_time;
      if t > self.pvp_bankara {
        // find new bankara schedule
        std::mem::swap(&mut self.pvp_bankara, &mut t);
        log::debug!(
          "cursor.pvp_bankara [{}] -> [{}]",
          t.with_timezone(&Local),
          self.pvp_bankara.with_timezone(&Local)
        );
        for s in bankara_schedules.nodes.into_iter() {
          if s.time_period.start_time > t {
            if let Some((challenge, open)) = s.match_setting.bankara_match_settings {
              collect_pvp(PvpMode::Challenge, s.time_period.clone(), challenge);
              collect_pvp(PvpMode::Open, s.time_period, open);
            }
          }
        }
      }
    }

    if let Some(s) = x_schedules.nodes.last() {
      let mut t = s.time_period.start_time;
      if t > self.pvp_x_match {
        // find new x match schedule
        std::mem::swap(&mut self.pvp_x_match, &mut t);
        log::debug!(
          "cursor.pvp_x_match [{}] -> [{}]",
          t.with_timezone(&Local),
          self.pvp_x_match.with_timezone(&Local)
        );
        for s in x_schedules.nodes.into_iter() {
          if s.time_period.start_time > t {
            if let Some(setting) = s.match_setting.x_match_setting {
              collect_pvp(PvpMode::X, s.time_period, setting);
            }
          }
        }
      }
    }

    if let Some(s) = fest_schedules.nodes.last() {
      let mut t = s.time_period.start_time;
      if t > self.pvp_fest {
        // find new x match schedule
        std::mem::swap(&mut self.pvp_fest, &mut t);
        log::debug!(
          "cursor.pvp_fest [{}] -> [{}]",
          t.with_timezone(&Local),
          self.pvp_fest.with_timezone(&Local)
        );
        for s in fest_schedules.nodes.into_iter() {
          if s.time_period.start_time > t {
            if let Some(setting) = s.match_setting.fest_match_setting {
              collect_pvp(PvpMode::Fest, s.time_period, setting);
            }
          }
        }
      }
    }

    if let Some(s) = coop_grouping_schedule.regular_schedules.nodes.last() {
      let mut t = s.time_period.start_time;
      if t > self.coop_normal {
        // find new coop schedule
        std::mem::swap(&mut self.coop_normal, &mut t);
        log::debug!(
          "cursor.coop_normal [{}] -> [{}]",
          t.with_timezone(&Local),
          self.coop_normal.with_timezone(&Local)
        );
        for s in coop_grouping_schedule.regular_schedules.nodes.into_iter() {
          if s.time_period.start_time > t {
            collect_coop(s);
          }
        }
      }
    }

    Ok((pvp, coop))
  }
}
