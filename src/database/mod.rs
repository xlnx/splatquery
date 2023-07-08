use std::{ops::Deref, path::Path};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;

use crate::Result;

pub mod pvp;
pub mod query;
pub mod user;

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

fn do_init(conn: &mut Connection) -> Result<(), rusqlite::Error> {
  conn.execute_batch(
    "BEGIN;
  
    CREATE TABLE IF NOT EXISTS
    users (
      id                  INTEGER PRIMARY KEY AUTOINCREMENT,
      auth_agent          TEXT NOT NULL,
      auth_uid            TEXT NOT NULL,
      name                TEXT,
      email               TEXT,
      picture             TEXT,
      UNIQUE ( auth_uid, auth_agent )
    );

    CREATE TABLE IF NOT EXISTS
    pvp_queries (
      id                  INTEGER PRIMARY KEY AUTOINCREMENT,
      uid                 INTEGER NOT NULL,
      created_time        DATETIME DEFAULT CURRENT_TIMESTAMP,
      modes               TINYINT NOT NULL,
      rules               TINYINT NOT NULL,
      includes            INT NOT NULL,
      excludes            INT NOT NULL,
      FOREIGN KEY ( uid ) REFERENCES users ( id ) ON DELETE CASCADE
    );

    CREATE INDEX IF NOT EXISTS pvp_queries_index
    ON pvp_queries ( uid );

    CREATE TABLE IF NOT EXISTS
    user_actions (
      id                  INTEGER PRIMARY KEY AUTOINCREMENT,
      uid                 INTEGER NOT NULL,
      act_agent           TEXT NOT NULL,
      act_config          TEXT NOT NULL,
      FOREIGN KEY ( uid ) REFERENCES users ( id ) ON DELETE CASCADE,
      UNIQUE ( uid, act_agent )
    );

    COMMIT;",
  )
}
