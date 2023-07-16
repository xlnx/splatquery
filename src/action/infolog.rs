use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{splatnet::Message, Result};

use super::{ActionAgent, ActionContext};

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoLogActionAgent {}

#[async_trait]
impl ActionAgent for InfoLogActionAgent {
  async fn emit(
    self: Arc<Self>,
    _ctx: Arc<ActionContext>,
    uid: i64,
    id: i64,
    msg: Arc<Message>,
  ) -> Result<()> {
    log::info!("uid[{}], id[{}] <- [{:?}]", uid, id, msg);
    Ok(())
  }
}
