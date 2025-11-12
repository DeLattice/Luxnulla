use luxnulla::CONFIG_DIR;
use rusqlite::{Connection, Result as RusqliteResult, params, params_from_iter};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::{Arc, Mutex};
use url::Url;

use crate::common::parsers::outbound::ExtraOutboundClientConfig;
use crate::http::services::model::xray_config::XrayOutboundClientConfig;
use crate::services::common::paginator::PaginationParams;

pub struct GroupModel {
    pub id: i32,
    pub name: String,
    pub subscribe_url: Option<Url>,
}

pub struct ConfigModel {
    pub id: i32,
    pub group_id: i32,
    pub data: String,
    pub extra: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Group {
    name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    sub_url: Option<Url>,

    configs: Vec<XrayOutboundModel>,
}

impl Group {
    pub fn new(name: String, sub_url: Option<Url>, configs: Vec<XrayOutboundModel>) -> Self {
        Self {
            name,
            sub_url,
            configs,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrayOutboundModel {
    pub id: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<ExtraOutboundClientConfig>,

    #[serde(flatten)]
    pub config: XrayOutboundClientConfig,
}

impl XrayOutboundModel {
    pub fn new(
        id: i32,
        extra: Option<ExtraOutboundClientConfig>,
        config: XrayOutboundClientConfig,
    ) -> Self {
        Self { id, extra, config }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedConfigs {
    pub configs: Vec<XrayOutboundModel>,
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
                name TEXT NOT NULL,
                sub_url TEXT NULL
            )",
            [],
        )
        .expect("Failed to create 'groups' table.");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS configs (
                id INTEGER PRIMARY KEY,
                group_id INTEGER NOT NULL,
                extra TEXT NOT NULL,
                data TEXT NOT NULL,
                FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE
            )",
            [],
        )
        .expect("Failed to create 'configs' table.");

        let sq = Arc::new(Mutex::new(conn));

        Self { sq }
    }

    pub fn create_group(
        &self,
        name: &str,
        sub_url: Option<Url>,
    ) -> Result<GroupModel, StorageError> {
        let mut conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let tx = conn.transaction()?;

        let id = tx
            .prepare("INSERT INTO groups (name, sub_url) VALUES (?1, ?2) RETURNING id")?
            .insert(params![name, sub_url.as_ref().map(|url| url.as_str())])?;

        tx.commit()?;

        Ok(GroupModel {
            id: id as i32,
            name: name.to_string(),
            subscribe_url: sub_url,
        })
    }

    pub fn update_group(
        &self,
        id: i32,
        name: Option<&str>,
        sub_url: Option<&Url>,
    ) -> Result<GroupModel, StorageError> {
        let mut conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let tx = conn.transaction()?;

        let group = tx
            .prepare("SELECT * FROM groups WHERE id = ?1")?
            .query_row(params![id], |row| {
                Ok(GroupModel {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    subscribe_url: row.get(2)?,
                })
            })?;

        let name = name.unwrap_or_else(|| &group.name);
        let sub_url = sub_url.or_else(|| group.subscribe_url.as_ref());

        tx.execute(
            "UPDATE groups SET name = ?1, sub_url = ?2 WHERE id = ?3",
            params![name, sub_url, id],
        )?;

        tx.commit()?;

        Ok(GroupModel {
            id,
            name: name.to_string(),
            subscribe_url: sub_url.map(|v| v.to_owned()),
        })
    }

    pub fn get_paginated_group_configs(
        &self,
        group_id: &i32,
        pagination: &PaginationParams,
    ) -> Result<Option<PaginatedConfigs>, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;

        let group_id_result: RusqliteResult<i64> =
            conn.query_row("SELECT id FROM groups WHERE id = ?1", [group_id], |row| {
                row.get(0)
            });

        let group_id = match group_id_result {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(StorageError::GroupNotFound(group_id.to_string()));
            }
            Err(e) => return Err(e.into()),
        };

        let total_items: i64 = conn.query_row(
            "SELECT COUNT(*) FROM configs WHERE group_id = ?1",
            params![group_id],
            |row| row.get(0),
        )?;

        let offset = (pagination.page as i64) * (pagination.limit as i64);
        let mut stmt = conn.prepare(
            "SELECT id, data, extra FROM configs WHERE group_id = ?1 LIMIT ?2 OFFSET ?3",
        )?;

        let configs = stmt
            .query_map(params![group_id, pagination.limit, offset], |row| {
                let config_id: i32 = row.get(0)?;
                let config_json: String = row.get(1)?;
                let config_extra: String = row.get(2)?;

                let extra = serde_json::from_str::<ExtraOutboundClientConfig>(&config_extra)
                    .expect("Failed to parse extra");

                serde_json::from_str::<XrayOutboundClientConfig>(&config_json)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })
                    .map(|config| XrayOutboundModel::new(config_id, Some(extra), config))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(PaginatedConfigs {
            configs,
            total_items,
        }))
    }

    pub fn get_group(&self, id: &i32) -> Result<Group, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;

        let group_metadata_result: RusqliteResult<(String, Option<String>)> = conn.query_row(
            "SELECT name, sub_url FROM groups WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        let (group_name, sub_url_str) = match group_metadata_result {
            Ok(data) => data,
            Err(e) => return Err(e.into()),
        };

        let sub_url: Option<Url> = sub_url_str.as_ref().and_then(|s| Url::parse(s).ok());

        let mut stmt = conn.prepare("SELECT id, extra, data FROM configs WHERE group_id = ?1")?;
        let configs = stmt
            .query_map([id], |row| {
                let config_id: i32 = row.get(0)?;
                let config_extra: String = row.get(1)?;
                let config_json: String = row.get(2)?;

                let extra = serde_json::from_str::<ExtraOutboundClientConfig>(&config_extra)
                    .expect("Failed to parse extra");

                serde_json::from_str(&config_json)
                    .map(|config| XrayOutboundModel::new(config_id, Some(extra), config))
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Group::new(group_name.to_string(), sub_url, configs))
    }

    pub fn get_configs_by_ids(
        &self,
        config_ids: &Vec<i32>,
    ) -> Result<Vec<XrayOutboundClientConfig>, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;

        if config_ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders = std::iter::repeat("?")
            .take(config_ids.len())
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            "SELECT id, data, extra FROM configs WHERE id IN ({})",
            placeholders
        );

        let mut stmt = conn.prepare(&query)?;
        let mut models = vec![];
        let mut rows = stmt.query(params_from_iter(config_ids.iter()))?;

        while let Some(row) = rows.next()? {
            let id: i32 = row.get(0)?;
            let config_json: String = row.get(1)?;
            let extra_json: String = row.get(2)?;

            let mut config = serde_json::from_str::<XrayOutboundClientConfig>(&config_json)
                .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
            let extra = serde_json::from_str::<Option<ExtraOutboundClientConfig>>(&extra_json)
                .map_err(|e| StorageError::DeserializationError(e.to_string()))?;

            config.tag = Some(id.to_string());

            models.push(XrayOutboundModel::new(id, extra, config));
        }

        Ok(models.into_iter().map(|model| model.config).collect())
    }

    pub fn list_groups(&self) -> Result<Vec<GroupModel>, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let mut stmt = conn.prepare("SELECT id, name, sub_url FROM groups")?;

        let groups = stmt
            .query_map([], |row| {
                Ok(GroupModel {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    subscribe_url: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(groups)
    }

    pub fn delete_group(&self, id: &i32) -> Result<bool, StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;

        let rows_affected = conn.execute("DELETE FROM groups WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }

    pub fn delete_all_groups(&self) -> Result<(), StorageError> {
        let conn = self.sq.lock().map_err(|_| StorageError::LockError)?;

        conn.execute("DELETE FROM groups", [])?;
        Ok(())
    }

    pub fn upsert_configs(
        &self,
        configs: Vec<(i32, XrayOutboundClientConfig)>,
    ) -> Result<Vec<XrayOutboundModel>, StorageError> {
        let mut conn = self.sq.lock().map_err(|_| StorageError::LockError)?;
        let tx = conn.transaction()?;

        let mut created_configs = Vec::new();

        {
            let mut stmt =
                tx.prepare("INSERT INTO configs (group_id, data) VALUES (?1, ?2) RETURNING id")?;

            for (group_id, config) in configs {
                tx.execute("DELETE FROM configs WHERE group_id = ?1", params![group_id])?;

                let config_json = serde_json::to_string(&config)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                let mut rows = stmt.query(params![group_id, config_json])?;
                if let Some(row) = rows.next()? {
                    let id: i32 = row.get(0)?;
                    created_configs.push(XrayOutboundModel::new(id, None, config));
                }
            }
        }

        tx.commit()?;
        Ok(created_configs)
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
