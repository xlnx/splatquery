use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct GearResponse {
  pub data: GearData,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct GearData {
  pub gesotown: GesoTown,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct GesoTown {
  #[serde(rename = "pickupBrand")]
  pub pickup_brand: PickupBrand,

  #[serde(rename = "limitedGears")]
  pub limited_gears: Vec<Gear>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct PickupBrand {
  #[serde(rename = "saleEndTime")]
  pub sale_end_time: String,

  #[serde(rename = "brandGears")]
  pub brand_gears: Vec<Gear>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Gear {
  pub id: String,

  pub price: i32,

  #[serde(rename = "saleEndTime")]
  pub sale_end_time: String,

  pub gear: Gear1,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Gear1 {
  #[serde(rename = "__splatoon3ink_id")]
  pub splatoon3ink_id: String,

  #[serde(rename = "__typename")]
  pub typename: String,

  pub name: String,

  pub brand: Brand,

  #[serde(rename = "primaryGearPower")]
  pub primary_gear_power: GearPower,

  #[serde(rename = "additionalGearPowers")]
  pub additional_gear_powers: Vec<GearPower>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct GearPower {
  #[serde(rename = "__splatoon3ink_id")]
  pub splatoon3ink_id: String,

  pub name: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Brand {
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
    let gears: GearResponse = serde_json::from_reader(reader).unwrap();
    assert_eq!(gears.data.gesotown.pickup_brand.brand_gears.len(), 3);
    assert_eq!(
      gears.data.gesotown.pickup_brand.brand_gears[0],
      Gear {
        id: String::from("U2FsZUdlYXItMF8xNjg2Nzg3MjAwXzA="),
        sale_end_time: String::from("2023-06-16T00:00:00Z"),
        price: 8000,
        gear: Gear1 {
          splatoon3ink_id: String::from("8a06264363dc442e"),
          typename: String::from("HeadGear"),
          name: String::from("Squidbeak Shield"),
          brand: Brand {
            id: String::from("QnJhbmQtMTc="),
            name: String::from("Toni Kensa")
          },
          primary_gear_power: GearPower {
            splatoon3ink_id: String::from("1d855c39cfd4d1ad"),
            name: String::from("Sub Resistance Up")
          },
          additional_gear_powers: vec![
            GearPower {
              splatoon3ink_id: String::from("cef7771e1562e6f9"),
              name: String::from("Unknown")
            },
            GearPower {
              splatoon3ink_id: String::from("cef7771e1562e6f9"),
              name: String::from("Unknown")
            },
          ]
        }
      }
    );
  }
}
