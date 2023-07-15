use std::sync::Arc;

use jsonwebtoken::Algorithm;
use serde::Deserialize;

#[cfg(feature = "renderer")]
use crate::renderer::RendererConfig;
use crate::{
  action::config::ActionAgentsConfig, database::DatabaseConfig, splatnet::SplatNetConfig, Result,
};

use super::auth::AuthAgentMap;
#[cfg(feature = "api-geoip2")]
use super::geoip2::GeoIp2Config;

#[derive(Deserialize)]
pub struct Config {
  pub http: HttpConfig,
  pub database: DatabaseConfig,
  #[serde(default)]
  pub splatnet: SplatNetConfig,
  pub auth: AuthConfig,
  #[serde(default)]
  pub actions: ActionAgentsConfig,
  #[serde(default)]
  pub geoip2: Option<GeoIp2Config>,
  #[cfg(feature = "renderer")]
  pub renderer: RendererConfig,
}

#[derive(Deserialize)]
pub struct HttpConfig {
  #[serde(default = "default_port")]
  pub port: u16,
  pub tls: TlsConfig,
  #[serde(default)]
  pub allow_origins: Vec<String>,
  pub cname: Option<String>,
}

fn default_port() -> u16 {
  443
}

#[derive(Deserialize)]
pub struct TlsConfig {
  pub pem: String,
  pub key: String,
}

// auth config
#[derive(Deserialize)]
pub struct AuthConfig {
  #[serde(default)]
  pub agents: AuthAgentsConfig,
  pub token: TokenConfig,
}

#[derive(Deserialize, Default)]
pub struct AuthAgentsConfig {
  #[cfg(feature = "api-auth-google")]
  pub google: Option<crate::api::auth::google::GoogleAuthAgent>,
}

#[derive(Deserialize)]
pub struct TokenConfig {
  #[serde(default = "default_token_algorithm")]
  pub algorithm: Algorithm,
  pub secret: String,
  #[serde(default = "default_token_expire_days")]
  pub expire_days: i64,
}

fn default_token_algorithm() -> Algorithm {
  Algorithm::HS256
}

fn default_token_expire_days() -> i64 {
  90
}

impl AuthAgentsConfig {
  pub fn collect(self) -> Result<Arc<AuthAgentMap>> {
    let mut auths = AuthAgentMap::new();
    #[cfg(feature = "api-auth-google")]
    if let Some(agent) = self.google {
      auths.insert("google", Arc::new(agent));
    }
    if auths.is_empty() {
      log::warn!("at least one auth agent should be specified");
    }
    Ok(Arc::new(auths))
  }
}
