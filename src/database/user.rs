use std::str::FromStr;

use r2d2_sqlite::rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

use super::{Language, TimeZone};

const DAY_HRS_MAX: i64 = (1i64 << 48) - 1;

#[derive(Debug)]
pub struct CreateUserRequest<'a> {
  pub auth_agent: &'a str,
  pub auth_uid: &'a str,
  pub name: Option<&'a str>,
  pub email: Option<&'a str>,
  pub picture: Option<&'a str>,
  pub language: Option<Language>,
  pub time_zone: Option<TimeZone>,
  // js can't represent integer > 2^53
  // https://stackoverflow.com/a/2983294/9602625
  pub day_hrs: Option<(i64, i64)>,
}

pub trait CreateUser {
  fn create_user(&self, request: CreateUserRequest) -> Result<bool>;
}

#[derive(Debug)]
pub struct LookupUserIdRequest<'a> {
  pub auth_agent: &'a str,
  pub auth_uid: &'a str,
}

pub trait LookupUserId {
  fn lookup_user_id(&self, request: LookupUserIdRequest) -> Result<i64>;
}

#[derive(Serialize, Deserialize)]
pub struct UserSettings {
  pub language: Option<Language>,
  pub time_zone: Option<TimeZone>,
  pub day_hrs: Option<(i64, i64)>,
}

pub trait ListUserSettings {
  fn list_user_settings(&self, uid: i64) -> Result<UserSettings>;
}

pub trait UpdateUserSettings {
  fn update_user_settings(&self, uid: i64, settings: &UserSettings) -> Result<()>;
}

impl CreateUser for Connection {
  fn create_user(&self, request: CreateUserRequest) -> Result<bool> {
    let CreateUserRequest {
      auth_agent,
      auth_uid,
      name,
      email,
      picture,
      language,
      time_zone,
      day_hrs,
    } = request;
    let language = language.unwrap_or(Language::EnUs).to_string();
    let time_zone = time_zone.unwrap_or(TimeZone::Jst).to_string();
    let (day_hrs_0, day_hrs_1) = day_hrs.unwrap_or((DAY_HRS_MAX, DAY_HRS_MAX));
    let n = self
      .prepare_cached(
        "
        INSERT OR IGNORE
        INTO users ( auth_agent, auth_uid, name, email, picture, language, time_zone, day_hrs_0, day_hrs_1 )
        VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9 )
        ",
      )?
      .execute((
        &auth_agent,
        &auth_uid,
        &name,
        &email,
        &picture,
        &language,
        &time_zone,
        &day_hrs_0,
        &day_hrs_1,
      ))?;
    Ok(n > 0)
  }
}

impl LookupUserId for Connection {
  fn lookup_user_id(&self, request: LookupUserIdRequest) -> Result<i64> {
    self
      .prepare_cached(
        "
        SELECT id 
        FROM users
        WHERE auth_uid = ?1 AND auth_agent = ?2
        ",
      )?
      .query_row((&request.auth_uid, &request.auth_agent), |row| row.get(0))
      .map_err(|err| match err {
        r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows => Error::Unauthorized,
        _ => Error::SqliteError(err),
      })
  }
}

impl ListUserSettings for Connection {
  fn list_user_settings(&self, uid: i64) -> Result<UserSettings> {
    let mut stmt = self.prepare_cached(
      "
      SELECT language, time_zone, day_hrs_0, day_hrs_1
      FROM users
      WHERE id = ?1
      ",
    )?;
    let s: (String, String, _, _) = stmt.query_row((&uid,), |row| {
      Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;
    let language =
      Some(Language::from_str(&s.0).map_err(|err| Error::InternalServerError(Box::new(err)))?);
    let time_zone =
      Some(TimeZone::from_str(&s.1).map_err(|err| Error::InternalServerError(Box::new(err)))?);
    Ok(UserSettings {
      language,
      time_zone,
      day_hrs: Some((s.2, s.3)),
    })
  }
}

impl UpdateUserSettings for Connection {
  fn update_user_settings(&self, uid: i64, settings: &UserSettings) -> Result<()> {
    let UserSettings {
      language,
      time_zone,
      day_hrs,
    } = settings;
    let language = language.map(|e| e.to_string());
    let time_zone = time_zone.map(|e| e.to_string());
    let mut stmt = self.prepare_cached(
      "
      UPDATE users
      SET 
        language = coalesce(?2, language), 
        time_zone = coalesce(?3, time_zone), 
        day_hrs_0 = coalesce(?4, day_hrs_0), 
        day_hrs_1 = coalesce(?5, day_hrs_1)
      WHERE id = ?1
      ",
    )?;
    let n = stmt.execute((
      &uid,
      &language,
      &time_zone,
      &day_hrs.map(|e| e.0),
      &day_hrs.map(|e| e.1),
    ))?;
    if n == 0 {
      Err(Error::Unauthorized)
    } else {
      Ok(())
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::database::Database;

  use super::*;

  #[tokio::test]
  async fn test_simple() {
    let db = Database::new_in_memory().unwrap();

    let conn = db.get().unwrap();
    let auth_agent = "mock_auth_agent";
    let u1 = conn
      .create_user(CreateUserRequest {
        auth_agent,
        auth_uid: "u1",
        name: None,
        email: None,
        picture: None,
        language: None,
        time_zone: None,
        day_hrs: None,
      })
      .unwrap();
    assert!(u1);
    let u2 = conn
      .create_user(CreateUserRequest {
        auth_agent,
        auth_uid: "u1",
        name: None,
        email: None,
        picture: None,
        language: None,
        time_zone: None,
        day_hrs: None,
      })
      .unwrap();
    assert!(!u2);
    let u3 = conn
      .create_user(CreateUserRequest {
        auth_agent,
        auth_uid: "u2",
        name: None,
        email: None,
        picture: None,
        language: None,
        time_zone: None,
        day_hrs: None,
      })
      .unwrap();
    assert!(u3);

    let uid = conn
      .lookup_user_id(LookupUserIdRequest {
        auth_agent,
        auth_uid: "u1",
      })
      .unwrap();
  }
}
