use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::Result;

use super::ActionAgentMap;

#[derive(Serialize, Deserialize, Default)]
pub struct ActionAgentsConfig {
  #[cfg(feature = "webpush")]
  pub webpush: Option<super::webpush::WebPushActionAgentConfig>,
}

impl ActionAgentsConfig {
  pub fn collect(self) -> Result<ActionAgentMap> {
    let mut actions = ActionAgentMap::new();
    #[cfg(feature = "webpush")]
    if let Some(agent) = self.webpush {
      actions.insert("webpush", Arc::new(agent.collect()?));
    }
    Ok(actions)
  }
}
