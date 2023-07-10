use rusqlite::Connection;
use serde::Deserialize;

use crate::Result;

#[derive(Deserialize)]
pub struct ToggleActionRequest {
  pub active: bool,
}

pub trait ToggleAction {
  fn toggle_action(&self, uid: i64, agent: &str, request: ToggleActionRequest) -> Result<()>;
}

impl ToggleAction for Connection {
  fn toggle_action(&self, uid: i64, agent: &str, request: ToggleActionRequest) -> Result<()> {
    let ToggleActionRequest { active } = request;
    let mut stmt = self.prepare_cached(
      "
      INSERT OR REPLACE INTO user_actions ( uid, act_agent, act_active )
      VALUES ( ?1, ?2, ?3 )
      ",
    )?;
    stmt.execute((&uid, &agent, &active))?;
    Ok(())
  }
}

pub trait ListAction {
  fn list_action(&self, uid: i64) -> Result<Vec<String>>;
}

impl ListAction for Connection {
  fn list_action(&self, uid: i64) -> Result<Vec<String>> {
    let mut stmt = self.prepare_cached(
      "
      SELECT act_agent
      FROM user_actions
      WHERE uid = ?1 AND act_active
      ",
    )?;
    let iter = stmt.query_map((&uid,), |row| Ok(row.get(0)?))?;
    let li = itertools::process_results(iter, |iter| iter.collect())?;
    Ok(li)
  }
}
