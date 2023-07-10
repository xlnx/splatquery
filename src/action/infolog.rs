use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{database::Database, splatnet::Message, Result};

use super::ActionAgent;

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoLogActionAgent {}

#[async_trait]
impl ActionAgent for InfoLogActionAgent {
  async fn send(self: Arc<Self>, _db: Database, msg: Message, uids: &[i64]) -> Result<()> {
    for uid in uids.iter() {
      log::info!("uid[{}] <- [{:?}]", uid, msg);
    }
    Ok(())
  }
}
