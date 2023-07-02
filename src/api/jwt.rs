use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{
  decode, encode, errors::Result, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
  Validation,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Clone)]
pub struct Agent(Arc<InnerAgent>);

struct InnerAgent {
  algorithm: Algorithm,
  header: Header,
  validation: Validation,
  validation_insecure: Validation,
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  decoding_key_insecure: DecodingKey,
}

#[derive(Serialize, Deserialize)]
struct PayloadWithExp<T> {
  #[serde(flatten)]
  payload: T,
  exp: i64,
}

impl std::fmt::Debug for Agent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("jwt::Agent")
      .field(&self.0.algorithm)
      .finish()
  }
}

impl Agent {
  pub fn new(algorithm: Algorithm, secret: &str) -> Self {
    let mut validation_insecure = Validation::new(Algorithm::HS256);
    validation_insecure.validate_exp = false;
    validation_insecure.required_spec_claims.clear();
    validation_insecure.insecure_disable_signature_validation();
    Agent(Arc::new(InnerAgent {
      algorithm,
      header: Header::new(algorithm),
      validation: Validation::new(algorithm),
      validation_insecure,
      encoding_key: EncodingKey::from_secret(secret.as_bytes()),
      decoding_key: DecodingKey::from_secret(secret.as_bytes()),
      decoding_key_insecure: DecodingKey::from_secret(&[]),
    }))
  }

  pub fn encode<T: Serialize>(&self, payload: &T, expiration: &Duration) -> Result<String> {
    encode(
      &self.0.header,
      &PayloadWithExp {
        payload: &payload,
        exp: (Utc::now() + expiration.clone()).timestamp(),
      },
      &self.0.encoding_key,
    )
  }

  pub fn decode<T: DeserializeOwned>(&self, jwt: &str) -> Result<T> {
    let data: TokenData<PayloadWithExp<T>> =
      decode(&jwt, &self.0.decoding_key, &self.0.validation)?;
    Ok(data.claims.payload)
  }

  pub fn decode_insecure<T: DeserializeOwned>(&self, jwt: &str) -> Result<T> {
    let data = decode(
      &jwt,
      &self.0.decoding_key_insecure,
      &self.0.validation_insecure,
    )?;
    Ok(data.claims)
  }
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

  use serde_json::{json, Value};

  use super::*;

  #[test]
  fn test_simple() {
    let agent = Agent::new(Algorithm::HS256, "");
    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let payload: BTreeMap<String, Value> = agent.decode_insecure(jwt).unwrap();
    assert_eq!(payload["sub"], json!("1234567890"));
    assert_eq!(payload["name"], json!("John Doe"));
    assert_eq!(payload["iat"], json!(1516239022));
    let jwt = agent.encode(&payload, &Duration::weeks(1)).unwrap();
    let payload: BTreeMap<String, Value> = agent.decode(&jwt).unwrap();
    assert_eq!(payload["sub"], json!("1234567890"));
    assert_eq!(payload["name"], json!("John Doe"));
    assert_eq!(payload["iat"], json!(1516239022));
  }
}
