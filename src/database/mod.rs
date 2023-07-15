use std::{ops::Deref, path::Path};

use chrono::{DateTime, FixedOffset};
use r2d2::Pool;
use r2d2_sqlite::{rusqlite::Connection, SqliteConnectionManager};
use serde::Deserialize;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use strum_macros::EnumIter;

use crate::Result;

pub mod action;
pub mod pvp;
pub mod query;
pub mod user;

#[derive(
  Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize_enum_str, Deserialize_enum_str, EnumIter,
)]
#[serde(rename_all = "lowercase")]
pub enum Language {
  EnUs,
}

#[derive(
  Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize_enum_str, Deserialize_enum_str, EnumIter,
)]
#[serde(rename_all = "lowercase")]
pub enum TimeZone {
  Jst = 9,
  Pt = -7,
  Cest = 2,
  Cst = 8,
}

impl TimeZone {
  pub fn convert<T>(self, t: DateTime<T>) -> DateTime<FixedOffset>
  where
    T: chrono::TimeZone,
  {
    t.with_timezone(&FixedOffset::east_opt(self as i32 * 3600).unwrap())
  }
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
  pub path: String,
}

impl DatabaseConfig {
  pub fn collect(self) -> Result<Database> {
    Database::new_from_file(self.path)
  }
}

#[derive(Clone)]
pub struct Database(Pool<SqliteConnectionManager>);

impl Database {
  pub fn new_in_memory() -> Result<Database> {
    // TODO: maybe any flags later
    let manager = SqliteConnectionManager::memory().with_init(do_init);
    Ok(Database(Pool::new(manager)?))
  }

  pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Database> {
    // TODO: maybe any flags later
    let manager = SqliteConnectionManager::file(path).with_init(do_init);
    Ok(Database(Pool::new(manager)?))
  }
}

impl Deref for Database {
  type Target = Pool<SqliteConnectionManager>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

fn do_init(conn: &mut Connection) -> Result<(), r2d2_sqlite::rusqlite::Error> {
  conn.execute("PRAGMA foreign_keys = ON", ())?;
  conn.execute_batch(
    "BEGIN;
  
    CREATE TABLE IF NOT EXISTS
    users (
      id                  INTEGER PRIMARY KEY AUTOINCREMENT,
      created_time        DATETIME DEFAULT CURRENT_TIMESTAMP,
      auth_agent          TEXT NOT NULL,
      auth_uid            TEXT NOT NULL,
      name                TEXT,
      email               TEXT,
      picture             TEXT,
      language            TEXT NOT NULL,
      time_zone           TEXT NOT NULL,
      day_hrs_0           INTEGER NOT NULL,   /* jst wd [0,4), 12 bits for each day  */
      day_hrs_1           INTEGER NOT NULL,   /* [4,7) */
      UNIQUE ( auth_uid, auth_agent )
    );

    CREATE TABLE IF NOT EXISTS
    pvp_queries (
      id                  INTEGER PRIMARY KEY AUTOINCREMENT,
      uid                 INTEGER NOT NULL,
      created_time        DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
      modes               TINYINT NOT NULL,
      rules               TINYINT NOT NULL,
      includes            INT NOT NULL,
      excludes            INT NOT NULL,
      FOREIGN KEY ( uid ) REFERENCES users ( id ) ON DELETE CASCADE
    );

    CREATE INDEX IF NOT EXISTS pvp_queries_index
    ON pvp_queries ( uid );

    CREATE TABLE IF NOT EXISTS
    user_action_agents (
      id                  INTEGER PRIMARY KEY AUTOINCREMENT,
      uid                 INTEGER NOT NULL,
      act_agent           TEXT NOT NULL,
      act_active          TINYINT NOT NULL,
      FOREIGN KEY ( uid ) REFERENCES users ( id ) ON DELETE CASCADE,
      UNIQUE ( uid, act_agent )
    );

    CREATE INDEX IF NOT EXISTS user_action_agents_index
    ON user_action_agents ( uid );

    CREATE TABLE IF NOT EXISTS
    user_actions (
      id                  INTEGER PRIMARY KEY AUTOINCREMENT,
      uid                 INTEGER NOT NULL,
      aid                 INTEGER NOT NULL,
      rx_pvp              INTEGER NOT NULL DEFAULT 0,
      rx_event            INTEGER NOT NULL DEFAULT 0,
      rx_coop             INTEGER NOT NULL DEFAULT 0,
      rx_coop_ex          INTEGER NOT NULL DEFAULT 0,
      rx_gear             INTEGER NOT NULL DEFAULT 0,
      rx_gear_brand       INTEGER NOT NULL DEFAULT 0,
      FOREIGN KEY ( aid ) REFERENCES user_action_agents ( id ) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS
    webpush_ext_info (
      id                  INTEGER UNIQUE NOT NULL,
      uid                 INTEGER NOT NULL,
      endpoint            TEXT NOT NULL,
      p256dh              TEXT NOT NULL,
      auth                TEXT NOT NULL,
      browser             TEXT,
      device              TEXT,
      os                  TEXT,
      FOREIGN KEY ( id ) REFERENCES user_actions ( id ) ON DELETE CASCADE,
      UNIQUE ( endpoint, uid )
    );

    COMMIT;",
  )
}
