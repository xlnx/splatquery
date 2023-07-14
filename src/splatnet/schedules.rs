use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct SchedulesResponse {
  pub data: SchedulesData,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct SchedulesData {
  #[serde(rename = "regularSchedules")]
  pub regular_schedules: ScheduleList<PVPSchedule<RegularMatchSetting>>,

  #[serde(rename = "bankaraSchedules")]
  pub bankara_schedules: ScheduleList<PVPSchedule<BankaraMatchSetting>>,

  #[serde(rename = "xSchedules")]
  pub x_schedules: ScheduleList<PVPSchedule<XMatchSetting>>,

  #[serde(rename = "eventSchedules")]
  pub event_schedules: ScheduleList<EventSchedule>,

  #[serde(rename = "festSchedules")]
  pub fest_schedules: ScheduleList<PVPSchedule<FestMatchSetting>>,

  #[serde(rename = "coopGroupingSchedule")]
  pub coop_grouping_schedule: CoopGroupingSchedule,
  // #[serde(rename = "vsStages")]
  // pub vs_stages: VSStages,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RegularMatchSetting {
  #[serde(rename = "regularMatchSetting")]
  pub regular_match_setting: Option<PVPMatchSetting>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct BankaraMatchSetting {
  #[serde(rename = "bankaraMatchSettings")]
  pub bankara_match_settings: Option<(PVPMatchSetting, PVPMatchSetting)>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct XMatchSetting {
  #[serde(rename = "xMatchSetting")]
  pub x_match_setting: Option<PVPMatchSetting>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct FestMatchSetting {
  #[serde(rename = "festMatchSetting")]
  pub fest_match_setting: Option<PVPMatchSetting>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(bound = "for<'de2> T: Deserialize<'de2>")]
pub struct ScheduleList<T>
where
  for<'de2> T: Deserialize<'de2> + PartialEq + Eq,
{
  pub nodes: Vec<T>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(bound = "for<'de2> T: Deserialize<'de2>")]
pub struct PVPSchedule<T>
where
  for<'de2> T: Deserialize<'de2> + PartialEq + Eq,
{
  #[serde(flatten)]
  pub time_period: TimePeriod,

  #[serde(flatten)]
  pub match_setting: T,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct EventSchedule {
  #[serde(rename = "timePeriods")]
  pub time_periods: Vec<TimePeriod>,

  #[serde(rename = "leagueMatchSetting")]
  pub league_match_setting: LeagueMatchSetting,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct TimePeriod {
  #[serde(rename = "startTime")]
  pub start_time: String,

  #[serde(rename = "endTime")]
  pub end_time: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct LeagueMatchSetting {
  #[serde(rename = "leagueMatchEvent")]
  pub league_match_event: LeagueMatchEvent,

  #[serde(flatten)]
  pub pvp_match_setting: PVPMatchSetting,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct LeagueMatchEvent {
  pub id: String,

  #[serde(rename = "leagueMatchEventId")]
  pub league_match_event_id: String,

  pub name: String,

  pub desc: String,

  pub regulation: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct PVPMatchSetting {
  #[serde(rename = "vsStages")]
  pub pvp_stages: Vec<PVPStage>,

  #[serde(rename = "vsRule")]
  pub pvp_rule: PVPRule,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct PVPRule {
  pub id: String,

  pub rule: String,

  pub name: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct PVPStage {
  pub id: String,

  #[serde(rename = "vsStageId")]
  pub pvp_stage_id: u32,

  pub name: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct CoopGroupingSchedule {
  #[serde(rename = "regularSchedules")]
  pub regular_schedules: ScheduleList<CoopNormalSchedule>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct CoopNormalSchedule {
  #[serde(flatten)]
  pub time_period: TimePeriod,

  pub setting: CoopNormalSetting,

  #[serde(rename = "__splatoon3ink_king_salmonid_guess")]
  pub splatoon3ink_king_salmonid_guess: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct CoopNormalSetting {
  #[serde(rename = "coopStage")]
  pub coop_stage: CoopStage,

  pub weapons: Vec<CoopWeapon>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct CoopStage {
  pub id: String,

  pub name: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct CoopWeapon {
  #[serde(rename = "__splatoon3ink_id")]
  pub splatoon3ink_id: String,

  pub name: String,
}

#[cfg(test)]
mod test {
  use std::{fs::File, io::BufReader, path::Path};

  use super::*;

  #[test]
  fn test_parse_schedules_response() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("resources/test/splat3ink_schedules_response.json");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let schedules: SchedulesResponse = serde_json::from_reader(reader).unwrap();
    assert_eq!(schedules.data.x_schedules.nodes.len(), 12);
    assert_eq!(
      schedules.data.x_schedules.nodes[0],
      PVPSchedule::<XMatchSetting> {
        time_period: TimePeriod {
          start_time: String::from("2023-06-15T16:00:00Z"),
          end_time: String::from("2023-06-15T18:00:00Z")
        },
        match_setting: XMatchSetting {
          x_match_setting: Some(PVPMatchSetting {
            pvp_stages: vec![
              PVPStage {
                id: String::from("VnNTdGFnZS0xNA=="),
                pvp_stage_id: 14,
                name: String::from("Sturgeon Shipyard")
              },
              PVPStage {
                id: String::from("VnNTdGFnZS0xOA=="),
                pvp_stage_id: 18,
                name: String::from("Manta Maria")
              }
            ],
            pvp_rule: PVPRule {
              id: String::from("VnNSdWxlLTI="),
              rule: String::from("LOFT"),
              name: String::from("Tower Control")
            }
          })
        }
      }
    );
    assert_eq!(
      schedules
        .data
        .coop_grouping_schedule
        .regular_schedules
        .nodes
        .len(),
      5
    );
    assert_eq!(
      schedules
        .data
        .coop_grouping_schedule
        .regular_schedules
        .nodes[0],
      CoopNormalSchedule {
        time_period: TimePeriod {
          start_time: String::from("2023-06-15T08:00:00Z"),
          end_time: String::from("2023-06-17T00:00:00Z")
        },
        setting: CoopNormalSetting {
          coop_stage: CoopStage {
            id: String::from("Q29vcFN0YWdlLTI="),
            name: String::from("Sockeye Station"),
          },
          weapons: vec![
            CoopWeapon {
              splatoon3ink_id: String::from("49171e6de78e50c7"),
              name: String::from("Splattershot Jr.")
            },
            CoopWeapon {
              splatoon3ink_id: String::from("09465cbd66e15c68"),
              name: String::from("Splat Dualies")
            },
            CoopWeapon {
              splatoon3ink_id: String::from("b0343d4f4b600e95"),
              name: String::from("Jet Squelcher")
            },
            CoopWeapon {
              splatoon3ink_id: String::from("aae42b6ef1b5090d"),
              name: String::from("Hydra Splatling")
            },
          ]
        },
        splatoon3ink_king_salmonid_guess: String::from("Cohozuna"),
      }
    );
  }
}
