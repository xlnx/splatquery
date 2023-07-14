use chrono::{DateTime, Utc};

use crate::BoxError;

use super::{
  gear,
  schedules::{self, CoopNormalSchedule},
  GearType, PVPMode,
};

#[derive(Debug)]
pub struct GearSpiderItem {
  pub sale_end_time: String,
  pub id: String,
  pub splatoon3ink_id: String,
  pub gear_type: GearType,
  pub brand: String,
  pub price: i32,
  pub primary_gear_power: String,
  pub additional_gear_powers: i32,
}

#[derive(Debug, Clone)]
pub struct PVPSpiderItem {
  pub start_time: String,
  pub end_time: String,
  pub rule: String,
  pub stages: Vec<u32>,
  pub mode: PVPMode,
}

#[derive(Debug)]
pub struct CoopSpiderItem {
  pub start_time: String,
  pub end_time: String,
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
    let json: gear::GearResponse = response.json().await?;
    self.do_update_gear(json).await
  }

  pub async fn update_schedules(
    &mut self,
  ) -> Result<(Vec<PVPSpiderItem>, Vec<CoopSpiderItem>), BoxError> {
    let url = "https://splatoon3.ink/data/schedules.json";
    let response = reqwest::get(url).await?;
    log::debug!("GET [{}] -> {}", url, response.status().as_u16());
    let json: schedules::SchedulesResponse = response.json().await?;
    self.do_update_schedules(json).await
  }

  async fn do_update_gear(
    &mut self,
    response: gear::GearResponse,
  ) -> Result<Vec<GearSpiderItem>, BoxError> {
    let gear::GesoTown {
      pickup_brand,
      limited_gears,
    } = response.data.gesotown;

    let mut gears = vec![];
    let mut collect = |e: gear::Gear| {
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

    let sale_end_time = DateTime::parse_from_rfc3339(&pickup_brand.sale_end_time)?.into();
    if sale_end_time > self.gear_pickup_brand {
      // found new pickup brand
      let t = self.gear_pickup_brand;
      self.gear_pickup_brand = sale_end_time;
      log::debug!(
        "cursor.gear_pickup_brand [{}] -> [{}]",
        t,
        self.gear_pickup_brand
      );
      for gear in pickup_brand.brand_gears.into_iter() {
        collect(gear);
      }
    }

    if let Some(gear) = limited_gears.last() {
      let sale_end_time = DateTime::parse_from_rfc3339(&gear.sale_end_time)?.into();
      if sale_end_time > self.gear_limited {
        // found new limited gears
        let t = self.gear_limited;
        self.gear_limited = sale_end_time;
        log::debug!("cursor.gear_limited [{}] -> [{}]", t, self.gear_limited);
        for gear in limited_gears.into_iter() {
          let t1 = DateTime::parse_from_rfc3339(&gear.sale_end_time)?;
          if t1 > t {
            collect(gear);
          }
        }
      }
    }

    Ok(gears)
  }

  async fn do_update_schedules(
    &mut self,
    response: schedules::SchedulesResponse,
  ) -> Result<(Vec<PVPSpiderItem>, Vec<CoopSpiderItem>), BoxError> {
    let schedules::SchedulesData {
      regular_schedules,
      bankara_schedules,
      x_schedules,
      fest_schedules,
      coop_grouping_schedule,
      ..
    } = response.data;

    let mut pvp = vec![];
    let mut collect_pvp =
      |mode: PVPMode, time_period: schedules::TimePeriod, setting: schedules::PVPMatchSetting| {
        pvp.push(PVPSpiderItem {
          start_time: time_period.start_time,
          end_time: time_period.end_time,
          rule: setting.pvp_rule.id,
          stages: setting
            .pvp_stages
            .into_iter()
            .map(|e| e.pvp_stage_id)
            .collect(),
          mode,
        });
      };

    let mut coop = vec![];
    let mut collect_coop = |schedule: CoopNormalSchedule| {
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
      let mut t = DateTime::parse_from_rfc3339(&s.time_period.start_time)?.into();
      if t > self.pvp_regular {
        // find new turf-war schedule
        std::mem::swap(&mut self.pvp_regular, &mut t);
        log::debug!("cursor.pvp_regular [{}] -> [{}]", t, self.pvp_regular);
        for s in regular_schedules.nodes.into_iter() {
          let t1 = DateTime::parse_from_rfc3339(&s.time_period.start_time)?;
          if t1 > t {
            if let Some(setting) = s.match_setting.regular_match_setting {
              collect_pvp(PVPMode::TurfWar, s.time_period, setting);
            }
          }
        }
      }
    }

    if let Some(s) = bankara_schedules.nodes.last() {
      let mut t = DateTime::parse_from_rfc3339(&s.time_period.start_time)?.into();
      if t > self.pvp_bankara {
        // find new bankara schedule
        std::mem::swap(&mut self.pvp_bankara, &mut t);
        log::debug!("cursor.pvp_bankara [{}] -> [{}]", t, self.pvp_bankara);
        for s in bankara_schedules.nodes.into_iter() {
          let t1 = DateTime::parse_from_rfc3339(&s.time_period.start_time)?;
          if t1 > t {
            if let Some((challenge, open)) = s.match_setting.bankara_match_settings {
              collect_pvp(PVPMode::Challenge, s.time_period.clone(), challenge);
              collect_pvp(PVPMode::Open, s.time_period, open);
            }
          }
        }
      }
    }

    if let Some(s) = x_schedules.nodes.last() {
      let mut t = DateTime::parse_from_rfc3339(&s.time_period.start_time)?.into();
      if t > self.pvp_x_match {
        // find new x match schedule
        std::mem::swap(&mut self.pvp_x_match, &mut t);
        log::debug!("cursor.pvp_x_match [{}] -> [{}]", t, self.pvp_x_match);
        for s in x_schedules.nodes.into_iter() {
          let t1 = DateTime::parse_from_rfc3339(&s.time_period.start_time)?;
          if t1 > t {
            if let Some(setting) = s.match_setting.x_match_setting {
              collect_pvp(PVPMode::X, s.time_period, setting);
            }
          }
        }
      }
    }

    if let Some(s) = fest_schedules.nodes.last() {
      let mut t = DateTime::parse_from_rfc3339(&s.time_period.start_time)?.into();
      if t > self.pvp_fest {
        // find new x match schedule
        std::mem::swap(&mut self.pvp_x_match, &mut t);
        log::debug!("cursor.pvp_fest [{}] -> [{}]", t, self.pvp_fest);
        for s in fest_schedules.nodes.into_iter() {
          let t1 = DateTime::parse_from_rfc3339(&s.time_period.start_time)?;
          if t1 > t {
            if let Some(setting) = s.match_setting.fest_match_setting {
              collect_pvp(PVPMode::Fest, s.time_period, setting);
            }
          }
        }
      }
    }

    if let Some(s) = coop_grouping_schedule.regular_schedules.nodes.last() {
      let mut t = DateTime::parse_from_rfc3339(&s.time_period.start_time)?.into();
      if t > self.coop_normal {
        // find new coop schedule
        std::mem::swap(&mut self.coop_normal, &mut t);
        log::debug!("cursor.coop_normal [{}] -> [{}]", t, self.coop_normal);
        for s in coop_grouping_schedule.regular_schedules.nodes.into_iter() {
          let t1 = DateTime::parse_from_rfc3339(&s.time_period.start_time)?;
          if t1 > t {
            collect_coop(s);
          }
        }
      }
    }

    Ok((pvp, coop))
  }
}
