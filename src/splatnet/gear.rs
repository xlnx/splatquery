use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawGearResponse {
  pub data: RawGearData,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawGearData {
  pub gesotown: RawGesoTown,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawGesoTown {
  #[serde(rename = "pickupBrand")]
  pub pickup_brand: RawPickupBrand,

  #[serde(rename = "limitedGears")]
  pub limited_gears: Vec<RawGear>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawPickupBrand {
  #[serde(rename = "saleEndTime")]
  #[serde(deserialize_with = "super::iso8601::parse")]
  pub sale_end_time: DateTime<Utc>,

  #[serde(rename = "brandGears")]
  pub brand_gears: Vec<RawGear>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawGear {
  pub id: String,

  pub price: i32,

  #[serde(rename = "saleEndTime")]
  #[serde(deserialize_with = "super::iso8601::parse")]
  pub sale_end_time: DateTime<Utc>,

  pub gear: RawGear1,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawGear1 {
  #[serde(rename = "__splatoon3ink_id")]
  pub splatoon3ink_id: String,

  #[serde(rename = "__typename")]
  pub typename: String,

  pub name: String,

  pub brand: RawBrand,

  #[serde(rename = "primaryGearPower")]
  pub primary_gear_power: RawGearPower,

  #[serde(rename = "additionalGearPowers")]
  pub additional_gear_powers: Vec<RawGearPower>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawGearPower {
  #[serde(rename = "__splatoon3ink_id")]
  pub splatoon3ink_id: String,

  pub name: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct RawBrand {
  pub id: String,

  pub name: String,
}

#[cfg(test)]
mod tests {
  use std::{fs::File, io::BufReader, path::Path};

  use super::*;

  #[test]
  fn test_parse_gear_response() {
    let path =
      Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/test/splat3ink_gear_response.json");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let gears: RawGearResponse = serde_json::from_reader(reader).unwrap();
    assert_eq!(gears.data.gesotown.pickup_brand.brand_gears.len(), 3);
    assert_eq!(
      gears.data.gesotown.pickup_brand.brand_gears[0],
      RawGear {
        id: String::from("U2FsZUdlYXItMF8xNjg2Nzg3MjAwXzA="),
        sale_end_time: DateTime::parse_from_rfc3339("2023-06-16T00:00:00Z")
          .unwrap()
          .into(),
        price: 8000,
        gear: RawGear1 {
          splatoon3ink_id: String::from("8a06264363dc442e"),
          typename: String::from("HeadGear"),
          name: String::from("Squidbeak Shield"),
          brand: RawBrand {
            id: String::from("QnJhbmQtMTc="),
            name: String::from("Toni Kensa")
          },
          primary_gear_power: RawGearPower {
            splatoon3ink_id: String::from("1d855c39cfd4d1ad"),
            name: String::from("Sub Resistance Up")
          },
          additional_gear_powers: vec![
            RawGearPower {
              splatoon3ink_id: String::from("cef7771e1562e6f9"),
              name: String::from("Unknown")
            },
            RawGearPower {
              splatoon3ink_id: String::from("cef7771e1562e6f9"),
              name: String::from("Unknown")
            },
          ]
        }
      }
    );
  }
}
