use std::{fs::File, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;
use web_push::{
  ContentEncoding, PartialVapidSignatureBuilder, SubscriptionInfo, SubscriptionKeys,
  VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

use crate::{
  database::Database,
  splatnet::{
    i18n::{EnUs, I18N},
    Message, PVPRule,
  },
  Error, Result,
};

use super::ActionAgent;

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

#[async_trait]
impl ActionAgent for WebPushActionAgent {
  async fn send(self: Arc<Self>, db: Database, msg: Message, uids: &[i64]) -> Result<()> {
    let mut subs = Vec::new();
    {
      let conn = db.get()?;
      let mut stmt = conn.prepare_cached(
        "
        SELECT endpoint, p256dh, auth 
        FROM action_webpush
        WHERE uid = ?1
        ",
      )?;
      for uid in uids.iter() {
        let iter = stmt.query_map((&uid,), |row| {
          Ok(SubscriptionInfo {
            endpoint: row.get(0)?,
            keys: SubscriptionKeys {
              p256dh: row.get(1)?,
              auth: row.get(2)?,
            },
          })
        })?;
        for sub in iter {
          subs.push(sub?);
        }
      }
    }
    if subs.is_empty() {
      return Ok(());
    }
    let payload = match msg {
      Message::PVP(item) => {
        let i18n = EnUs();
        let mode = i18n.get_pvp_mode_name(item.mode);
        let rule = i18n.get_pvp_rule_name(PVPRule::from_base64(&item.rule));
        let stages: Vec<_> = item
          .stages
          .iter()
          .map(|id| i18n.get_pvp_stage_name(*id))
          .collect();
        let title = format!("{} - {}", rule, mode);
        let body = format!("[{}] & [{}]", stages[0], stages[1]);
        let tag = base64::encode(format!("pvp-[{}]-[{}]", item.mode, item.start_time));
        let timestamp = DateTime::parse_from_rfc3339(&item.start_time)
          .unwrap_or_else(|_| DateTime::<FixedOffset>::MAX_UTC.into())
          .timestamp_millis();
        serde_json::to_vec(&json!({
          "title": title,
          "options": {
            "body": body,
            "tag": tag,
            "timestamp": timestamp,
          }
        }))
        .map_err(|err| Error::InternalServerError(Box::new(err)))?
      }
    };
    for sub in subs.into_iter() {
      self.send_one(&payload, &sub).await?;
    }
    Ok(())
  }
}

impl WebPushActionAgent {
  async fn send_one(&self, payload: &[u8], sub: &SubscriptionInfo) -> Result<()> {
    let vapid = self
      .vapid
      .clone()
      .add_sub_info(&sub)
      .build()
      .map_err(|err| Error::InternalServerError(Box::new(err)))?;

    let mut builder =
      WebPushMessageBuilder::new(&sub).map_err(|err| Error::InternalServerError(Box::new(err)))?;
    builder.set_payload(ContentEncoding::Aes128Gcm, &payload);
    builder.set_vapid_signature(vapid);

    let message = builder
      .build()
      .map_err(|err| Error::InternalServerError(Box::new(err)))?;

    self
      .client
      .send(message)
      .await
      .map_err(|err| Error::InternalServerError(Box::new(err)))?;

    log::debug!("sent [{}] bytes -> [{}]", payload.len(), sub.endpoint);
    Ok(())
  }
}

#[derive(Serialize, Deserialize)]
pub struct WebPushSubscribeRequest {
  #[serde(flatten)]
  pub sub: SubscriptionInfo,
  pub browser: Option<String>,
  pub device: Option<String>,
  pub os: Option<String>,
}

pub trait WebPushSubscribe {
  fn webpush_subscribe(&self, uid: i64, request: WebPushSubscribeRequest) -> Result<()>;
}

impl WebPushSubscribe for Connection {
  fn webpush_subscribe(&self, uid: i64, request: WebPushSubscribeRequest) -> Result<()> {
    let WebPushSubscribeRequest {
      sub,
      browser,
      device,
      os,
    } = request;
    let mut stmt = self.prepare_cached(
      "
      INSERT INTO action_webpush ( uid, endpoint, p256dh, auth, browser, device, os )
      VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7 )
      ",
    )?;
    let n = stmt.execute((
      &uid,
      &sub.endpoint,
      &sub.keys.p256dh,
      &sub.keys.auth,
      &browser,
      &device,
      &os,
    ))?;
    if n == 0 {
      Err(Error::SqliteError(rusqlite::Error::QueryReturnedNoRows))
    } else {
      Ok(())
    }
  }
}

#[derive(Deserialize)]
pub struct WebPushDismissRequest {
  pub endpoint: String,
}

pub trait WebPushDismiss {
  fn webpush_dismiss(&self, uid: i64, request: WebPushDismissRequest) -> Result<()>;
}

impl WebPushDismiss for Connection {
  fn webpush_dismiss(&self, uid: i64, request: WebPushDismissRequest) -> Result<()> {
    let WebPushDismissRequest { endpoint } = request;
    let mut stmt = self.prepare_cached(
      "
      DELETE FROM action_webpush
      WHERE uid = ?1 AND endpoint = ?2
      ",
    )?;
    let n = stmt.execute((&uid, &endpoint))?;
    if n == 0 {
      Err(Error::SqliteError(rusqlite::Error::QueryReturnedNoRows))
    } else {
      Ok(())
    }
  }
}

#[derive(Serialize)]
pub struct WebPushListResponse {
  pub endpoint: String,
  pub browser: Option<String>,
  pub device: Option<String>,
  pub os: Option<String>,
}

pub trait WebPushList {
  fn webpush_list(&self, uid: i64) -> Result<Vec<WebPushListResponse>>;
}

impl WebPushList for Connection {
  fn webpush_list(&self, uid: i64) -> Result<Vec<WebPushListResponse>> {
    let mut stmt = self.prepare_cached(
      "
      SELECT endpoint, browser, device, os
      FROM action_webpush
      WHERE uid = ?1
      ",
    )?;
    let iter = stmt.query_map((&uid,), |row| {
      Ok(WebPushListResponse {
        endpoint: row.get(0)?,
        browser: row.get(1)?,
        device: row.get(2)?,
        os: row.get(3)?,
      })
    })?;
    let li = itertools::process_results(iter, |iter| iter.collect())?;
    Ok(li)
  }
}
