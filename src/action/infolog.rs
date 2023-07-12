use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{database::Database, splatnet::Message, Result};

use super::ActionAgent;

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoLogActionAgent {}

#[async_trait]
impl ActionAgent for InfoLogActionAgent {
  async fn emit(self: Arc<Self>, _db: Database, id: i64, msg: Arc<Message>) -> Result<()> {
    log::info!("id[{}] <- [{:?}]", id, msg);
    Ok(())
  }
}
