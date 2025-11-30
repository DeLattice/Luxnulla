use elux::DB_FILE_NAME;
use rusqlite::{Connection, Result as SqliteResult};

use crate::utils::config::AppPaths;

pub struct DbConnection {
    conn: Connection,
}

impl DbConnection {
    pub fn new() -> SqliteResult<Self> {
        let db_path = &AppPaths::get().config_dir.join(DB_FILE_NAME);

        let conn = Connection::open(db_path)?;
        Ok(DbConnection { conn })
    }

    pub fn init_schema(&self) -> SqliteResult<()> {
        self.conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS groups (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                subscribe_url TEXT NULL
            );

            CREATE TABLE IF NOT EXISTS configs (
                id INTEGER PRIMARY KEY,
                group_id INTEGER NOT NULL,
                extra TEXT NOT NULL,
                data TEXT NOT NULL,
                FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
            );",
        )?;
        Ok(())
    }
}
