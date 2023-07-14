use chrono::{DateTime, Utc};
use serde::{de, Deserializer};

pub fn parse<'de, D>(data: D) -> Result<DateTime<Utc>, D::Error>
where
  D: Deserializer<'de>,
{
  let s: String = de::Deserialize::deserialize(data)?;
  let t =
    DateTime::parse_from_rfc3339(&s).map_err(|err| de::Error::custom(format!("{:?}", err)))?;
  Ok(t.into())
}
