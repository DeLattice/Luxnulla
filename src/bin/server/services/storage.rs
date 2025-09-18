use luxnulla::CONFIG_DIR;
use rusqlite::{params, Connection, Result as RusqliteResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::{Arc, Mutex};

use crate::http::services::model::xray_config::XrayOutboundClientConfig;
use crate::services::common::paginator::PaginationParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub configs: Vec<XrayOutboundClientConfig>,
}

impl Group {
    pub fn new(name: String, configs: Vec<XrayOutboundClientConfig>) -> Self {
        Self { name, configs }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedConfigs {
    pub configs: Vec<XrayOutboundClientConfig>,
    pub total_items: i64,
}

#[derive(Debug, Clone)]
pub struct StorageService {
    sq: Arc<Mutex<Connection>>,
}

impl StorageService {
    pub fn new() -> Self {
        let config_dir_path = dirs::config_dir().expect("Failed to find config directory.");
        let app_dir = config_dir_path.join(CONFIG_DIR);

        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)
                .unwrap_or_else(|e| panic!("Failed to create app config directory: {}", e));
        }

        let db_path = app_dir.join("storage.db");
        let conn = Connection::open(&db_path)
            .unwrap_or_else(|e| panic!("Failed to open database at {:?}: {}", db_path, e));

        conn.execute("PRAGMA foreign_keys = ON;", [])
            .expect("Failed to enable foreign keys.");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS groups (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )
        .expect("Failed to create 'groups' table.");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS configs (
                id INTEGER PRIMARY KEY,
                group_id INTEGER NOT NULL,
                config_data TEXT NOT NULL,
                FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE
            )",
            [],
        )
        .expect("Failed to create 'configs' table.");


        let sq = Arc::new(Mutex::new(conn));

        Self { sq }
    }

    pub fn group_exists(&self, name: &str) -> Result<bool, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare("SELECT 1 FROM groups WHERE name = ?1")?;
        let exists = stmt.exists([name])?;
        Ok(exists)
    }

    pub fn upsert_group(&self, group: Group) -> Result<(), StorageError> {
        let mut conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT OR IGNORE INTO groups (name) VALUES (?1)",
            params![&group.name],
        )?;

        let group_id: i64 = tx.query_row(
            "SELECT id FROM groups WHERE name = ?1",
            params![&group.name],
            |row| row.get(0),
        )?;

        tx.execute("DELETE FROM configs WHERE group_id = ?1", params![group_id])?;

        {
            let mut stmt = tx.prepare("INSERT INTO configs (group_id, config_data) VALUES (?1, ?2)")?;
            for config in &group.configs {
                let config_json = serde_json::to_string(config)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                stmt.execute(params![group_id, config_json])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_paginated_group_configs(
        &self,
        name: &str,
        pagination: &PaginationParams,
    ) -> Result<Option<PaginatedConfigs>, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;

        let group_id_result: RusqliteResult<i64> = conn.query_row(
            "SELECT id FROM groups WHERE name = ?1",
            [name],
            |row| row.get(0),
        );

        let group_id = match group_id_result {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err(StorageError::GroupNotFound(name.to_string())),
            Err(e) => return Err(e.into()),
        };

        let total_items: i64 = conn.query_row(
            "SELECT COUNT(*) FROM configs WHERE group_id = ?1",
            params![group_id],
            |row| row.get(0),
        )?;

        let offset = (pagination.page as i64) * (pagination.limit as i64);
        let mut stmt = conn.prepare(
            "SELECT config_data FROM configs WHERE group_id = ?1 LIMIT ?2 OFFSET ?3",
        )?;

        let configs_iter = stmt.query_map(params![group_id, pagination.limit, offset], |row| {
            let config_json: String = row.get(0)?;
            serde_json::from_str(&config_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
        })?;

        let mut configs = Vec::new();
        for config_result in configs_iter {
            configs.push(config_result?);
        }

        Ok(Some(PaginatedConfigs { configs, total_items }))
    }

    pub fn get_group(&self, name: &str) -> Result<Option<Group>, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;

        let group_id_result: RusqliteResult<i64> = conn.query_row(
            "SELECT id FROM groups WHERE name = ?1",
            [name],
            |row| row.get(0),
        );

        let group_id = match group_id_result {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let mut stmt = conn.prepare("SELECT config_data FROM configs WHERE group_id = ?1")?;
        let configs_iter = stmt.query_map([group_id], |row| {
            let config_json: String = row.get(0)?;
            serde_json::from_str(&config_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
        })?;

        let mut configs = Vec::new();
        for result in configs_iter {
            configs.push(result?);
        }

        Ok(Some(Group::new(name.to_string(), configs)))
    }

    pub fn list_group_names(&self) -> Result<Vec<String>, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare("SELECT name FROM groups")?;
        let names_iter = stmt.query_map([], |row| row.get(0))?;
        names_iter
            .collect::<RusqliteResult<Vec<String>>>()
            .map_err(Into::into)
    }

    pub fn delete_group(&self, name: &str) -> Result<bool, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;

        let rows_affected = conn.execute("DELETE FROM groups WHERE name = ?1", [name])?;
        Ok(rows_affected > 0)
    }

    pub fn delete_all_groups(&self) -> Result<(), StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;

        conn.execute("DELETE FROM groups", [])?;
        Ok(())
    }

    pub fn count_groups(&self) -> Result<usize, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM groups", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

impl Default for StorageService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Failed to acquire lock on storage")]
    LockError,

    #[error("Group '{0}' not found")]
    GroupNotFound(String),

    #[error("Storage operation failed: {0}")]
    OperationFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> Self {
        StorageError::OperationFailed(err.to_string())
    }
}
