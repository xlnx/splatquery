use std::{fs::File, str::FromStr, sync::Arc};

use async_trait::async_trait;
use r2d2_sqlite::rusqlite::{Connection, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::json;
use web_push::{
  ContentEncoding, PartialVapidSignatureBuilder, SubscriptionInfo, SubscriptionKeys,
  VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

use crate::{
  database::{action::CreateAction, Language, TimeZone},
  renderer::RenderOptions,
  splatnet::{
    i18n::{EnUs, I18N},
    Message,
  },
  Error, Result,
};

use super::{ActionAgent, ActionContext};

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

#[derive(Serialize)]
pub struct WebPushExtInfo {
  pub endpoint: String,
  pub browser: Option<String>,
  pub device: Option<String>,
  pub os: Option<String>,
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
  fn get_ext_info(
    &self,
    conn: &Connection,
    id: i64,
  ) -> Result<Option<Box<dyn erased_serde::Serialize>>> {
    let mut stmt = conn.prepare_cached(
      "
      SELECT endpoint, browser, device, os
      FROM webpush_ext_info
      WHERE id = ?1
      ",
    )?;
    let info = stmt.query_row((&id,), |row| {
      Ok(WebPushExtInfo {
        endpoint: row.get(0)?,
        browser: row.get(1)?,
        device: row.get(2)?,
        os: row.get(3)?,
      })
    })?;
    Ok(Some(Box::new(info)))
  }

  async fn emit(
    self: Arc<Self>,
    ctx: Arc<ActionContext>,
    id: i64,
    msg: Arc<Message>,
  ) -> Result<()> {
    let (sub, os, language, time_zone): (_, String, String, String) = {
      let conn = ctx.database.get()?;
      let mut stmt = conn.prepare_cached(
        "
        SELECT endpoint, p256dh, auth, os, language, time_zone
        FROM webpush_ext_info
          INNER JOIN users ON users.id = uid
        WHERE webpush_ext_info.id = ?1
        ",
      )?;
      stmt.query_row((&id,), |row| {
        Ok((
          SubscriptionInfo {
            endpoint: row.get(0)?,
            keys: SubscriptionKeys {
              p256dh: row.get(1)?,
              auth: row.get(2)?,
            },
          },
          row.get(3)?,
          row.get(4)?,
          row.get(5)?,
        ))
      })?
    };
    let language =
      Language::from_str(&language).map_err(|err| Error::InternalServerError(Box::new(err)))?;
    let time_zone =
      TimeZone::from_str(&time_zone).map_err(|err| Error::InternalServerError(Box::new(err)))?;
    let payload = match msg.as_ref() {
      Message::PVP(item) => {
        let i18n = EnUs();
        let mode = i18n.get_pvp_mode_name(item.mode);
        let rule = i18n.get_pvp_rule_name(item.rule);
        let stages: Vec<_> = item
          .stages
          .iter()
          .map(|id| i18n.get_pvp_stage_name(*id))
          .collect();
        let title = format!("{} - {}", rule, mode);
        let body = format!("[{}] & [{}]", stages[0], stages[1]);
        let tag = base64::encode(format!("pvp-[{}]-[{}]", item.mode, item.start_time));
        let platform = if os.starts_with("Windows") {
          "pc"
        } else {
          "mobile"
        };
        let img_opts = RenderOptions {
          platform,
          language,
          time_zone,
        };
        let img_path = ctx
          .renderer
          .render_pvp(item, &img_opts)
          .map_err(|err| Error::InternalServerError(err))?;
        serde_json::to_vec(&json!({
          "title": title,
          "options": {
            "body": body,
            "image": format!("{}/{}", ctx.image_url, img_path),
            "icon": "https://splatquery.koishi.top/logo.svg",
            "silent": true,
            "tag": tag,
            "timestamp": item.start_time.timestamp_millis(),
          }
        }))
        .map_err(|err| Error::InternalServerError(Box::new(err)))?
      }
    };

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
  fn webpush_subscribe(&self, uid: i64, request: WebPushSubscribeRequest) -> Result<i64>;
}

impl<'a> WebPushSubscribe for Transaction<'a> {
  fn webpush_subscribe(&self, uid: i64, request: WebPushSubscribeRequest) -> Result<i64> {
    let id = self.create_action(uid, "webpush")?;
    let WebPushSubscribeRequest {
      sub,
      browser,
      device,
      os,
    } = request;
    let mut stmt = self.prepare_cached(
      "
      INSERT INTO webpush_ext_info ( id, uid, endpoint, p256dh, auth, browser, device, os )
      VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8 )
      ",
    )?;
    let n = stmt.execute((
      &id,
      &uid,
      &sub.endpoint,
      &sub.keys.p256dh,
      &sub.keys.auth,
      &browser,
      &device,
      &os,
    ))?;
    if n == 0 {
      Err(Error::SqliteError(
        r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows,
      ))
    } else {
      Ok(id)
    }
  }
}
