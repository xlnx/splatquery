use rusqlite::{Connection, Transaction};

use crate::Result;

pub trait CreateAction {
  fn create_action(&self, uid: i64, agent: &str) -> Result<i64>;
}

impl<'a> CreateAction for Transaction<'a> {
  fn create_action(&self, uid: i64, agent: &str) -> Result<i64> {
    self
      .prepare_cached(
        "
        INSERT OR IGNORE INTO user_action_agents ( uid, act_agent, act_active )
        VALUES ( ?1, ?2, 1 )
        ",
      )?
      .execute((&uid, &agent))?;
    let aid: i64 = self
      .prepare_cached(
        "
        SELECT id
        FROM user_action_agents
        WHERE uid = ?1 AND act_agent = ?2
        ",
      )?
      .query_row((&uid, &agent), |row| Ok(row.get(0)?))?;
    self
      .prepare_cached(
        "
        INSERT INTO user_actions ( aid, uid )
        VALUES ( ?1, ?2 )
        ",
      )?
      .execute((&aid, &uid))?;
    Ok(self.last_insert_rowid())
  }
}

pub trait DeleteAction {
  fn delete_action(&self, uid: i64, id: i64) -> Result<()>;
}

impl DeleteAction for Connection {
  fn delete_action(&self, uid: i64, id: i64) -> Result<()> {
    self
      .prepare_cached(
        "
        DELETE FROM user_actions
        WHERE uid = ?1 AND id = ?2
        ",
      )?
      .execute((&uid, &id))?;
    Ok(())
  }
}

pub trait ToggleAction {
  fn toggle_action(&self, uid: i64, agent: &str, active: bool) -> Result<()>;
}

impl ToggleAction for Connection {
  fn toggle_action(&self, uid: i64, agent: &str, active: bool) -> Result<()> {
    self
      .prepare_cached(
        "
        UPDATE user_action_agents
        SET act_active = ?3
        WHERE uid = ?1 AND act_agent = ?2
        ",
      )?
      .execute((&uid, &agent, &active))?;
    Ok(())
  }
}

pub struct ListActionResponse {
  pub id: i64,
  pub agent: String,
  pub active: bool,
}

pub trait ListAction {
  fn list_action(&self, uid: i64) -> Result<Vec<ListActionResponse>>;
}

impl ListAction for Connection {
  fn list_action(&self, uid: i64) -> Result<Vec<ListActionResponse>> {
    let mut stmt = self.prepare_cached(
      "
      SELECT user_actions.id, user_action_agents.act_agent, user_action_agents.act_active
      FROM user_action_agents
        INNER JOIN user_actions 
          ON user_action_agents.id = user_actions.aid
      WHERE user_action_agents.uid = ?1
      ",
    )?;
    let iter = stmt.query_map((uid,), |row| {
      Ok(ListActionResponse {
        id: row.get(0)?,
        agent: row.get(1)?,
        active: row.get(2)?,
      })
    })?;
    let li = itertools::process_results(iter, |iter| iter.collect())?;
    Ok(li)
  }
}
