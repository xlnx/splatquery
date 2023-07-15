use std::{ops::Deref, sync::Arc};

use maxminddb::Reader;
use serde::Deserialize;

use crate::{Error, Result};

#[derive(Deserialize)]
pub struct GeoIp2Config {
  pub mmdb_path: String,
}

#[derive(Clone)]
pub struct GeoIp2(Arc<Reader<Vec<u8>>>);

impl Deref for GeoIp2 {
  type Target = Reader<Vec<u8>>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl GeoIp2Config {
  pub fn collect(self) -> Result<GeoIp2> {
    let reader = maxminddb::Reader::open_readfile(self.mmdb_path)
      .map_err(|err| Error::InternalServerError(Box::new(err)))?;
    Ok(GeoIp2(Arc::new(reader)))
  }
}
