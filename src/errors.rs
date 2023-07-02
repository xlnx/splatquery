#[cfg(feature = "api")]
use axum::response::{IntoResponse, Response};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("internal error")]
  InternalServerError(BoxError),

  #[error("network error")]
  NetworkError(#[from] reqwest::Error),

  #[error("r2d2 error")]
  R2D2Error(#[from] r2d2::Error),

  #[error("sqlite error")]
  SqliteError(#[from] rusqlite::Error),

  #[cfg(feature = "api")]
  #[error("jwt error")]
  JwtError(#[from] jsonwebtoken::errors::Error),

  #[error("invalid parameter")]
  InvalidParameter(&'static str, String),

  #[cfg(feature = "api")]
  #[error("unauthorized")]
  Unauthorized,
}

#[cfg(feature = "api")]
impl IntoResponse for Error {
  fn into_response(self) -> Response {
    use http::StatusCode;
    let code = match self {
      Self::InternalServerError(err) => {
        log::error!("internal error: [{:?}]", err);
        StatusCode::INTERNAL_SERVER_ERROR
      }
      Self::NetworkError(err) => {
        log::error!("reqwest error treated as internal error: [{:?}]", err);
        StatusCode::INTERNAL_SERVER_ERROR
      }
      Self::R2D2Error(err) => {
        log::warn!("db connection pool timeout: [{:?}]", err);
        StatusCode::REQUEST_TIMEOUT
      }
      Self::SqliteError(err) => {
        use rusqlite::Error;
        match err {
          Error::QueryReturnedNoRows => {
            log::info!("entity not exist");
            StatusCode::BAD_REQUEST
          }
          _ => {
            log::error!("sqlite error treated as internal error: [{:?}]", err);
            StatusCode::INTERNAL_SERVER_ERROR
          }
        }
      }
      Self::JwtError(err) => {
        use jsonwebtoken::errors::ErrorKind;
        match err.kind() {
          ErrorKind::ExpiredSignature => {
            log::info!("token expired: [{:?}]", err);
            StatusCode::UNAUTHORIZED
          }
          _ => {
            log::error!("jwt error treated as internal error: [{:?}]", err);
            StatusCode::INTERNAL_SERVER_ERROR
          }
        }
      }
      Self::InvalidParameter(param, value) => {
        log::info!("invalid parameter `{}`: [{}]", param, value);
        StatusCode::BAD_REQUEST
      }
      Self::Unauthorized => {
        log::info!("unauthorized");
        StatusCode::BAD_REQUEST
      }
    };
    code.into_response()
  }
}
