use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawSchedulesResponse {
  pub data: RawSchedulesData,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawSchedulesData {
  #[serde(rename = "regularSchedules")]
  pub regular_schedules: RawScheduleList<RawPvpSchedule<RawRegularMatchSetting>>,

  #[serde(rename = "bankaraSchedules")]
  pub bankara_schedules: RawScheduleList<RawPvpSchedule<RawBankaraMatchSetting>>,

  #[serde(rename = "xSchedules")]
  pub x_schedules: RawScheduleList<RawPvpSchedule<RawXMatchSetting>>,

  #[serde(rename = "eventSchedules")]
  pub event_schedules: RawScheduleList<RawEventSchedule>,

  #[serde(rename = "festSchedules")]
  pub fest_schedules: RawScheduleList<RawPvpSchedule<RawFestMatchSetting>>,

  #[serde(rename = "coopGroupingSchedule")]
  pub coop_grouping_schedule: RawCoopGroupingSchedule,
  // #[serde(rename = "vsStages")]
  // pub vs_stages: VSStages,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawRegularMatchSetting {
  #[serde(rename = "regularMatchSetting")]
  pub regular_match_setting: Option<RawPvpMatchSetting>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawBankaraMatchSetting {
  #[serde(rename = "bankaraMatchSettings")]
  pub bankara_match_settings: Option<(RawPvpMatchSetting, RawPvpMatchSetting)>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawXMatchSetting {
  #[serde(rename = "xMatchSetting")]
  pub x_match_setting: Option<RawPvpMatchSetting>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawFestMatchSetting {
  #[serde(rename = "festMatchSetting")]
  pub fest_match_setting: Option<RawPvpMatchSetting>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(bound = "for<'de2> T: Deserialize<'de2>")]
pub struct RawScheduleList<T>
where
  for<'de2> T: Deserialize<'de2> + PartialEq + Eq,
{
  pub nodes: Vec<T>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(bound = "for<'de2> T: Deserialize<'de2>")]
pub struct RawPvpSchedule<T>
where
  for<'de2> T: Deserialize<'de2> + PartialEq + Eq,
{
  #[serde(flatten)]
  pub time_period: RawTimePeriod,

  #[serde(flatten)]
  pub match_setting: T,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawEventSchedule {
  #[serde(rename = "timePeriods")]
  pub time_periods: Vec<RawTimePeriod>,

  #[serde(rename = "leagueMatchSetting")]
  pub league_match_setting: RawLeagueMatchSetting,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct RawTimePeriod {
  #[serde(rename = "startTime")]
  #[serde(deserialize_with = "super::iso8601::parse")]
  pub start_time: DateTime<Utc>,

  #[serde(rename = "endTime")]
  #[serde(deserialize_with = "super::iso8601::parse")]
  pub end_time: DateTime<Utc>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawLeagueMatchSetting {
  #[serde(rename = "leagueMatchEvent")]
  pub league_match_event: RawLeagueMatchEvent,

  #[serde(flatten)]
  pub pvp_match_setting: RawPvpMatchSetting,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawLeagueMatchEvent {
  pub id: String,

  #[serde(rename = "leagueMatchEventId")]
  pub league_match_event_id: String,

  pub name: String,

  pub desc: String,

  pub regulation: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawPvpMatchSetting {
  #[serde(rename = "vsStages")]
  pub pvp_stages: Vec<RawPvpStage>,

  #[serde(rename = "vsRule")]
  pub pvp_rule: RawPvpRule,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawPvpRule {
  pub id: String,

  pub rule: String,

  pub name: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawPvpStage {
  pub id: String,

  #[serde(rename = "vsStageId")]
  pub pvp_stage_id: u32,

  pub name: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawCoopGroupingSchedule {
  #[serde(rename = "regularSchedules")]
  pub regular_schedules: RawScheduleList<RawCoopNormalSchedule>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawCoopNormalSchedule {
  #[serde(flatten)]
  pub time_period: RawTimePeriod,

  pub setting: RawCoopNormalSetting,

  #[serde(rename = "__splatoon3ink_king_salmonid_guess")]
  pub splatoon3ink_king_salmonid_guess: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawCoopNormalSetting {
  #[serde(rename = "coopStage")]
  pub coop_stage: RawCoopStage,

  pub weapons: Vec<RawCoopWeapon>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawCoopStage {
  pub id: String,

  pub name: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawCoopWeapon {
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
    let schedules: RawSchedulesResponse = serde_json::from_reader(reader).unwrap();
    assert_eq!(schedules.data.x_schedules.nodes.len(), 12);
    assert_eq!(
      schedules.data.x_schedules.nodes[0],
      RawPvpSchedule::<RawXMatchSetting> {
        time_period: RawTimePeriod {
          start_time: DateTime::parse_from_rfc3339("2023-06-15T16:00:00Z")
            .unwrap()
            .into(),
          end_time: DateTime::parse_from_rfc3339("2023-06-15T18:00:00Z")
            .unwrap()
            .into()
        },
        match_setting: RawXMatchSetting {
          x_match_setting: Some(RawPvpMatchSetting {
            pvp_stages: vec![
              RawPvpStage {
                id: String::from("VnNTdGFnZS0xNA=="),
                pvp_stage_id: 14,
                name: String::from("Sturgeon Shipyard")
              },
              RawPvpStage {
                id: String::from("VnNTdGFnZS0xOA=="),
                pvp_stage_id: 18,
                name: String::from("Manta Maria")
              }
            ],
            pvp_rule: RawPvpRule {
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
      RawCoopNormalSchedule {
        time_period: RawTimePeriod {
          start_time: DateTime::parse_from_rfc3339("2023-06-15T08:00:00Z")
            .unwrap()
            .into(),
          end_time: DateTime::parse_from_rfc3339("2023-06-17T00:00:00Z")
            .unwrap()
            .into()
        },
        setting: RawCoopNormalSetting {
          coop_stage: RawCoopStage {
            id: String::from("Q29vcFN0YWdlLTI="),
            name: String::from("Sockeye Station"),
          },
          weapons: vec![
            RawCoopWeapon {
              splatoon3ink_id: String::from("49171e6de78e50c7"),
              name: String::from("Splattershot Jr.")
            },
            RawCoopWeapon {
              splatoon3ink_id: String::from("09465cbd66e15c68"),
              name: String::from("Splat Dualies")
            },
            RawCoopWeapon {
              splatoon3ink_id: String::from("b0343d4f4b600e95"),
              name: String::from("Jet Squelcher")
            },
            RawCoopWeapon {
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
