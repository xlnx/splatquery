use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct CreateUserRequest<'a> {
  pub auth_agent: &'a str,
  pub auth_uid: &'a str,
  pub name: Option<&'a str>,
  pub email: Option<&'a str>,
  pub picture: Option<&'a str>,
}

pub trait CreateUser {
  fn create_user(&self, request: CreateUserRequest) -> Result<bool>;
}

#[derive(Debug)]
pub struct LookupUserRequest<'a> {
  pub auth_agent: &'a str,
  pub auth_uid: &'a str,
}

pub trait LookupUser {
  fn lookup_user(&self, request: LookupUserRequest) -> Result<i64>;
}

#[derive(Debug)]
pub struct UpdateUserActionRequest<'a> {
  pub uid: i64,
  pub act_agent: &'a str,
}

pub trait UpdateUserAction {
  fn update_user_action(&self, request: UpdateUserActionRequest) -> Result<()>;
}

impl CreateUser for Connection {
  fn create_user(&self, request: CreateUserRequest) -> Result<bool> {
    let n = self
      .prepare_cached(
        "
        INSERT OR IGNORE
        INTO users ( auth_agent, auth_uid, name, email, picture )
        VALUES ( ?1, ?2, ?3, ?4, ?5 )
        ",
      )?
      .execute((
        &request.auth_agent,
        &request.auth_uid,
        &request.name,
        &request.email,
        &request.picture,
      ))?;
    Ok(n > 0)
  }
}

impl LookupUser for Connection {
  fn lookup_user(&self, request: LookupUserRequest) -> Result<i64> {
    self
      .prepare_cached(
        "
        SELECT id 
        FROM users
        WHERE auth_uid = ?1 AND auth_agent = ?2
        ",
      )?
      .query_row((&request.auth_uid, &request.auth_agent), |row| row.get(0))
  }
}

impl UpdateUserAction for Connection {
  fn update_user_action(&self, request: UpdateUserActionRequest) -> Result<()> {
    self
      .prepare_cached(
        "
        INSERT OR REPLACE
        INTO user_actions ( uid, act_agent, act_active )
        VALUES ( ?1, ?2, 1 )
        ",
      )?
      .execute((&request.uid, &request.act_agent))?;
    Ok(())
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
      })
      .unwrap();
    assert!(u3);

    let uid = conn
      .lookup_user(LookupUserRequest {
        auth_agent,
        auth_uid: "u1",
      })
      .unwrap();

    let act_agent = "mock_act_agent";
    conn
      .update_user_action(UpdateUserActionRequest { uid, act_agent })
      .unwrap();
  }
}
