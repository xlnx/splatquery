use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
  async_trait,
  extract::{ConnectInfo, Path, State},
  response::{AppendHeaders, IntoResponse},
  Json,
};
use http::header::AUTHORIZATION;
#[cfg(feature = "api-geoip2")]
use maxminddb::geoip2::country::Country;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
  api::UserInfo,
  database::{
    user::{CreateUser, CreateUserRequest},
    Language, TimeZone,
  },
  Error, Result,
};

use super::state::{AppState, InnerAppState};

#[cfg(feature = "api-auth-google")]
pub mod google;

#[derive(Deserialize, Debug)]
pub struct AuthRequest {
  pub code: String,
  pub redirect_uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthUserInfo {
  pub id: String,
  pub name: Option<String>,
  pub email: Option<String>,
  pub picture: Option<String>,
}

pub type AuthAgentMap = HashMap<&'static str, Arc<dyn AuthAgent>>;

#[async_trait]
pub trait AuthAgent: Send + Sync {
  async fn oauth2(&self, request: &AuthRequest) -> Result<AuthUserInfo>;
}

pub async fn oauth2(
  Path(agent_type): Path<String>,
  State(state): State<AppState>,
  #[cfg(feature = "api-geoip2")] ConnectInfo(addr): ConnectInfo<SocketAddr>,
  Json(request): Json<AuthRequest>,
) -> Result<impl IntoResponse> {
  let InnerAppState {
    auths,
    db,
    jwt,
    auth_expiration,
    geoip2,
    ..
  } = state.0.as_ref();

  // find the specified agent
  let agent = auths
    .get(agent_type.as_str())
    .ok_or_else(|| Error::InvalidParameter("agent_type", agent_type.clone()))?;
  log::debug!("incoming auth request: [{:?}]", request);

  // send oauth2 request to auth server
  let auth = agent.oauth2(&request).await?;

  let mut language = None;
  let mut time_zone = None;

  #[cfg(feature = "api-geoip2")]
  if let Some(geoip2) = geoip2 {
    if let Ok(country) = geoip2.lookup::<Country>(addr.ip()) {
      // https://dev.maxmind.com/geoip/docs/databases/city-and-country
      // https://www.geonames.org/
      (time_zone, language) = match country {
        Country {
          is_in_european_union: Some(true /* EU */),
          ..
        } => {
          log::debug!("{:?} -> [cest/enus]", country);
          (Some(TimeZone::Cest), Some(Language::EnUs))
        }
        Country {
          geoname_id: Some(1861060 /* JP */),
          ..
        } => {
          log::debug!("{:?} -> [jst/enus]", country);
          (Some(TimeZone::Jst), Some(Language::EnUs))
        }
        Country {
          geoname_id: Some(1814991 /* CHN */),
          ..
        } => {
          log::debug!("{:?} -> [cst/enus]", country);
          (Some(TimeZone::Cst), Some(Language::EnUs))
        }
        Country {
          geoname_id: Some(6252001 /* US */) | Some(6251999 /* CA */),
          ..
        } => {
          log::debug!("{:?} -> [pdt/enus]", country);
          (Some(TimeZone::Pt), Some(Language::EnUs))
        }
        _ => (None, None),
      };
    }
  }

  // store userinfo to db
  let ok = db.get()?.create_user(CreateUserRequest {
    auth_agent: &agent_type,
    auth_uid: &auth.id,
    name: auth.name.as_deref(),
    email: auth.email.as_deref(),
    picture: auth.picture.as_deref(),
    language,
    time_zone,
    day_hrs: None,
  })?;

  if ok {
    log::debug!("user created: [{:?}]", &auth);
  } else {
    log::debug!("user already exists: [{:?}]", (&agent_type, &auth.id));
  }

  // sign our jwt
  let jwt = jwt.encode(
    &UserInfo {
      agent: agent_type,
      id: auth.id,
    },
    &auth_expiration,
  )?;
  log::debug!("signed auth request: [{:?}], jwt: [{}]", request, jwt);

  Ok((
    // emit jwt
    AppendHeaders([(AUTHORIZATION, String::from("Bearer ") + &jwt)]),
    // emit userinfo
    serde_json::to_string(&json!({
      "name": auth.name,
      "email": auth.email,
      "picture": auth.picture,
    }))
    .map_err(|err| Error::InternalServerError(Box::new(err)))?,
  ))
}
