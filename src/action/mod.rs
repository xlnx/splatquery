use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use serde_json::Value;

use crate::errors::Result;

pub mod config;
pub mod infolog;
#[cfg(feature = "webpush")]
pub mod webpush;

pub type ActionAgentMap = HashMap<&'static str, Arc<dyn ActionAgent>>;

pub trait ActionAgent: std::fmt::Debug + Send + Sync {
  fn new_action(self: Arc<Self>, config: &str) -> Result<Box<dyn Action>>;
}

#[async_trait]
pub trait Action: Send + Sync {
  async fn emit(self: Box<Self>, item: &Value) -> Result<()>;
}
