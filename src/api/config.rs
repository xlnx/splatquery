use std::sync::Arc;

use jsonwebtoken::Algorithm;
use serde::{Deserialize, Serialize};

use crate::{action::config::ActionAgentsConfig, splatnet::SplatNetConfig, Result};

use super::auth::AuthAgentMap;

#[derive(Serialize, Deserialize)]
pub struct Config {
  #[serde(default = "default_port")]
  pub port: u16,
  pub cert: CertConfig,
  pub database: DatabaseConfig,
  #[serde(default)]
  pub splatnet: SplatNetConfig,
  pub auth: AuthConfig,
  #[serde(default)]
  pub actions: ActionAgentsConfig,
}

fn default_port() -> u16 {
  443
}

#[derive(Serialize, Deserialize)]
pub struct CertConfig {
  pub pem: String,
  pub key: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthConfig {
  #[serde(default)]
  pub agents: AuthAgentsConfig,
  pub token: TokenConfig,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AuthAgentsConfig {
  #[cfg(feature = "api-auth-google")]
  pub google: Option<crate::api::auth::google::GoogleAuthAgent>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenConfig {
  #[serde(default = "default_token_algorithm")]
  pub algorithm: Algorithm,
  pub secret: String,
  #[serde(default = "default_token_expire_days")]
  pub expire_days: i64,
}

#[derive(Serialize, Deserialize)]
pub struct DatabaseConfig {
  pub path: String,
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
