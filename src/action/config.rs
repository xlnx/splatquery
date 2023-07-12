use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::Result;

use super::{infolog::InfoLogActionAgent, ActionAgentMap};

#[derive(Serialize, Deserialize, Default)]
pub struct ActionAgentsConfig {
  pub infolog: Option<InfoLogActionAgent>,
  #[cfg(feature = "webpush")]
  pub webpush: Option<super::webpush::WebPushActionAgentConfig>,
}

impl ActionAgentsConfig {
  pub fn collect(self) -> Result<ActionAgentMap> {
    let mut actions = ActionAgentMap::new();
    if let Some(agent) = self.infolog {
      actions.insert("infolog", Arc::new(agent));
    }
    #[cfg(feature = "webpush")]
    if let Some(agent) = self.webpush {
      actions.insert("webpush", Arc::new(agent.collect()?));
    }
    if actions.is_empty() {
      log::warn!("at least one agent agent should be specified");
    }
    Ok(actions)
  }
}
