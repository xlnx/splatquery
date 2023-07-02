use std::sync::Arc;

use jsonwebtoken::Algorithm;
use serde::{Deserialize, Serialize};

use crate::{action::config::ActionAgentsConfig, errors::Result, splatnet::SplatNetConfig};

use super::auth::AuthAgentMap;

#[derive(Serialize, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub splatnet: SplatNetConfig,
  pub auth: AuthConfig,
  #[serde(default)]
  pub actions: ActionAgentsConfig,
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

fn default_token_algorithm() -> Algorithm {
  Algorithm::HS256
}

fn default_token_expire_days() -> i64 {
  90
}

impl AuthAgentsConfig {
  pub fn collect(self) -> Result<AuthAgentMap> {
    let mut auths = AuthAgentMap::new();
    #[cfg(feature = "api-auth-google")]
    if let Some(agent) = self.google {
      auths.insert("google", Arc::new(agent));
    }
    Ok(auths)
  }
}
