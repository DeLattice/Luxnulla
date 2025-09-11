use luxnulla::CONFIG_DIR;
use rusqlite::{Connection, Result as RusqliteResult, params};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::{Arc, Mutex};

use crate::http::services::model::xray_config::XrayClientOutboundConfig;
use crate::services::common::paginator::PaginationParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub configs: Vec<XrayClientOutboundConfig>,
}

impl Group {
    pub fn new(name: String, configs: Vec<XrayClientOutboundConfig>) -> Self {
        Self { name, configs }
    }
}

#[derive(Debug, Clone)]
pub struct StorageService {
    sq: Arc<Mutex<Connection>>,
}

impl StorageService {
    pub fn new() -> Self {
        let config_dir_path = dirs::config_dir().expect("Failed to find config directory.");
        let app_dir = config_dir_path.join(CONFIG_DIR);

        // Создаем директорию приложения, если она отсутствует
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)
                .unwrap_or_else(|e| panic!("Failed to create app config directory: {}", e));
        }

        let db_path = app_dir.join("storage.db");
        let conn = Connection::open(&db_path)
            .unwrap_or_else(|e| panic!("Failed to open database at {:?}: {}", db_path, e));

        conn.execute(
            "CREATE TABLE IF NOT EXISTS groups (
                name TEXT PRIMARY KEY NOT NULL,
                configs TEXT NOT NULL
            )",
            [],
        )
        .expect("Failed to create 'groups' table.");

        let sq = Arc::new(Mutex::new(conn));

        Self { sq }
    }

    pub fn group_exists(&self, name: &str) -> Result<bool, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare("SELECT 1 FROM groups WHERE name = ?1")?;
        let exists = stmt.exists([name])?;
        Ok(exists)
    }

    pub fn store_group(&self, group: Group) -> Result<(), StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let configs_json = serde_json::to_string(&group.configs)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        conn.execute(
            "INSERT OR REPLACE INTO groups (name, configs) VALUES (?1, ?2)",
            params![group.name, configs_json],
        )?;

        Ok(())
    }

    pub fn get_paginated_group_configs(
        &self,
        name: &str,
        pagination: &PaginationParams,
    ) -> Result<Option<Group>, StorageError> {
        if let Some(mut group) = self.get_group(name)? {
            let paginated_configs: Vec<XrayClientOutboundConfig> = group
                .configs
                .into_iter()
                .skip(pagination.page)
                .take(pagination.limit)
                .collect();

            group.configs = paginated_configs;
            Ok(Some(group))
        } else {
            Ok(None)
        }
    }

    pub fn get_group(&self, name: &str) -> Result<Option<Group>, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare("SELECT configs FROM groups WHERE name = ?1")?;

        match stmt.query_row([name], |row| row.get::<_, String>(0)) {
            Ok(configs_json) => {
                let configs = serde_json::from_str(&configs_json)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                Ok(Some(Group {
                    name: name.to_string(),
                    configs,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_all_groups(&self) -> Result<Vec<String>, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare("SELECT name FROM groups")?;

        let group_iter = stmt.query_map([], |row| {
            Ok(row.get(0)?)
        })?;

        group_iter
            .collect::<RusqliteResult<Vec<String>>>()
            .map_err(Into::into)
    }

    pub fn update_group_config(&self, group: Group) -> Result<bool, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let configs_json = serde_json::to_string(&group.configs)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let rows_affected = conn.execute(
            "UPDATE groups SET configs = ?1 WHERE name = ?2",
            params![configs_json, group.name],
        )?;

        Ok(rows_affected > 0)
    }

    pub fn delete_group(&self, name: &str) -> Result<bool, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let rows_affected = conn.execute("DELETE FROM groups WHERE name = ?1", [name])?;
        Ok(rows_affected > 0)
    }

    pub fn delete_all_group(&self) -> Result<(), StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        conn.execute("DELETE FROM groups", [])?;
        Ok(())
    }

    pub fn count_groups(&self) -> Result<usize, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM groups", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn list_group_names(&self) -> Result<Vec<String>, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare("SELECT name FROM groups")?;
        let names_iter = stmt.query_map([], |row| row.get(0))?;
        names_iter
            .collect::<RusqliteResult<Vec<String>>>()
            .map_err(Into::into)
    }

    pub fn upsert_group(&self, group: Group) -> Result<bool, StorageError> {
        let mut conn = self.sq.lock().map_err(|_| StorageError::LockError)?;

        let tx = conn.transaction()?;
        let existed: bool;

        {
            let mut stmt = tx.prepare_cached("SELECT 1 FROM groups WHERE name = ?1")?;
            existed = stmt.exists([&group.name])?;
        }

        {
            let configs_json = serde_json::to_string(&group.configs)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            let mut stmt =
                tx.prepare_cached("INSERT OR REPLACE INTO groups (name, configs) VALUES (?1, ?2)")?;
            stmt.execute(params![group.name, configs_json])?;
        }

        tx.commit()?;
        Ok(existed)
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
