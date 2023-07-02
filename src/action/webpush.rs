use std::{fs::File, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_push::{
  ContentEncoding, PartialVapidSignatureBuilder, SubscriptionInfo, VapidSignatureBuilder,
  WebPushClient, WebPushMessageBuilder,
};

use crate::{Error, Result};

use super::{Action, ActionAgent};

#[derive(Serialize, Deserialize)]
pub struct WebPushActionAgentConfig {
  pub private_pem_path: String,
}

impl WebPushActionAgentConfig {
  pub fn collect(self) -> Result<WebPushActionAgent> {
    let file = File::open(&self.private_pem_path).map_err(|err| {
      log::error!("{:?}", err);
      Error::InvalidParameter("webpush::private_pem", self.private_pem_path.clone())
    })?;
    let vapid = VapidSignatureBuilder::from_pem_no_sub(file).map_err(|err| {
      log::error!("{:?}", err);
      Error::InvalidParameter("webpush::private_pem", self.private_pem_path.clone())
    })?;
    let client = WebPushClient::new().map_err(|err| {
      log::error!("{:?}", err);
      Error::InternalServerError("create webpush client failed".into())
    })?;
    Ok(WebPushActionAgent { vapid, client })
  }
}

pub struct WebPushActionAgent {
  vapid: PartialVapidSignatureBuilder,
  client: WebPushClient,
}

impl std::fmt::Debug for WebPushActionAgent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("WebPushActionAgent").finish()
  }
}

impl ActionAgent for WebPushActionAgent {
  fn new_action(self: Arc<Self>, config: &str) -> Result<Box<dyn Action>> {
    let subscription_info =
      serde_json::from_str(config).map_err(|err| Error::InternalServerError(Box::new(err)))?;
    Ok(Box::new(WebPushAction {
      subscription_info,
      agent: self.clone(),
    }))
  }
}

#[derive(Serialize, Deserialize)]
pub struct WebPushActionConfig {
  #[serde(flatten)]
  pub subscription_info: SubscriptionInfo,
  pub browser: Option<String>,
  pub device: Option<String>,
  pub os: Option<String>,
}

pub struct WebPushAction {
  subscription_info: SubscriptionInfo,
  agent: Arc<WebPushActionAgent>,
}

#[async_trait]
impl Action for WebPushAction {
  async fn emit(self: Box<Self>, item: &Value) -> Result<()> {
    let vapid = self
      .agent
      .vapid
      .clone()
      .add_sub_info(&self.subscription_info)
      .build()
      .map_err(|err| Error::InternalServerError(Box::new(err)))?;

    // FIXME: replace with real msg
    // FIXME: convert item format
    let content = "Encrypted payload to be sent in the notification".as_bytes();

    let mut builder = WebPushMessageBuilder::new(&self.subscription_info)
      .map_err(|err| Error::InternalServerError(Box::new(err)))?;
    builder.set_payload(ContentEncoding::Aes128Gcm, content);
    builder.set_vapid_signature(vapid);

    let message = builder
      .build()
      .map_err(|err| Error::InternalServerError(Box::new(err)))?;

    self
      .agent
      .client
      .send(message)
      .await
      .map_err(|err| Error::InternalServerError(Box::new(err)))?;

    log::info!("send");
    Ok(())
  }
}
