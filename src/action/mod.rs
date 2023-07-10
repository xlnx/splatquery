use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::{database::Database, splatnet::Message, Result};

pub mod config;
pub mod infolog;
#[cfg(feature = "webpush")]
pub mod webpush;

pub type ActionAgentMap = HashMap<&'static str, Arc<dyn ActionAgent>>;

#[async_trait]
pub trait ActionAgent: std::fmt::Debug + Send + Sync {
  async fn send(self: Arc<Self>, db: Database, msg: Message, uids: &[i64]) -> Result<()>;
}
