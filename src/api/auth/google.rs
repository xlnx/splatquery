use axum::async_trait;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

use super::{AuthAgent, AuthRequest, AuthUserInfo};

const GOOGLE_OAUTH2_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_OAUTH2_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

#[derive(Serialize, Debug)]
struct FetchTokenRequest<'a> {
  code: &'a str,
  client_id: &'a str,
  client_secret: &'a str,
  redirect_uri: &'a str,
  grant_type: &'a str,
}

#[derive(Deserialize)]
struct FetchTokenResponse {
  access_token: String,
}

#[derive(Deserialize)]
struct FetchUserInfoResponse {
  id: String,
  email: String,
  given_name: String,
  family_name: String,
  picture: String,
}

#[derive(Serialize, Deserialize)]
pub struct GoogleAuthAgent {
  client_id: String,
  client_secret: String,
}

#[async_trait]
impl AuthAgent for GoogleAuthAgent {
  async fn oauth2(&self, AuthRequest { code, redirect_uri }: &AuthRequest) -> Result<AuthUserInfo> {
    let client = reqwest::Client::new();

    // prepare request body
    let request = FetchTokenRequest {
      code: &code,
      client_id: &self.client_id,
      client_secret: &self.client_secret,
      redirect_uri: &redirect_uri,
      grant_type: "authorization_code",
    };

    // send gauth request
    let response = client
      .post(GOOGLE_OAUTH2_TOKEN_URL)
      .json(&request)
      .send()
      .await?;

    // check gauth status
    if !response.status().is_success() {
      log::debug!(
        "bad gauth request: code [{}], error: [{}]",
        code,
        response.text().await?
      );
      return Err(Error::Unauthorized);
    }

    // fetch access token
    let response: FetchTokenResponse = response.json().await?;

    // fetch userinfo for user identity
    let response = client
      .get(GOOGLE_OAUTH2_USERINFO_URL)
      .query(&[
        ("alt", "json"),
        ("oauth_token", &response.access_token),
        ("fields", "id,email,picture,given_name,family_name"),
      ])
      .send()
      .await?;

    // check userinfo status
    if !response.status().is_success() {
      log::debug!("get userinfo failed: code [{}]", code);
      return Err(Error::Unauthorized);
    }

    // fetch userinfo
    let FetchUserInfoResponse {
      id,
      email,
      picture,
      given_name,
      family_name,
    } = response.json().await.map_err(|err| {
      log::debug!(
        "parse userinfo response failed, maybe due to insufficient auth scope: [{:?}]",
        err
      );
      Error::Unauthorized
    })?;

    // userinfo ok
    let name = given_name + " " + &family_name;
    log::debug!("login success: code: [{}] -> [{} ({})]", code, name, email);
    Ok(AuthUserInfo {
      id,
      name: Some(name),
      email: Some(email),
      picture: Some(picture),
    })
  }
}
