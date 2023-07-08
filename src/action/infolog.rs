use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Result;

use super::{Action, ActionAgent};

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoLogActionAgent {}

impl ActionAgent for InfoLogActionAgent {
  fn new_action(self: Arc<Self>, _: &str) -> Result<Box<dyn Action>> {
    Ok(Box::new(InfoLogAction()))
  }
}

pub struct InfoLogAction();

#[async_trait]
impl Action for InfoLogAction {
  async fn emit(self: Box<Self>, item: &Value) -> Result<()> {
    log::info!("{:?}", item);
    Ok(())
  }
}
