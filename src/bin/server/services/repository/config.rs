use rusqlite::{OptionalExtension, Result as SqliteResult, Transaction, params, params_from_iter};
use serde::{Deserialize, Serialize};

use crate::services::{common::paginator::PaginationParams, db::utils::repeat_vars};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigModel {
    pub id: i32,
    pub group_id: i32,
    pub data: String,
    pub extra: String,
}

impl ConfigModel {
    pub fn new(group_id: i32, data: String, extra: String) -> Self {
        Self {
            id: 0,
            group_id,
            data,
            extra,
        }
    }
}

pub struct ConfigRepository;

impl ConfigRepository {
    pub fn create(tx: &Transaction, config: &ConfigModel) -> SqliteResult<i32> {
        tx.execute(
            "INSERT INTO configs (group_id, data, extra) VALUES (?1, ?2, ?3)",
            params![config.group_id, &config.data, &config.extra],
        )?;

        Ok(tx.last_insert_rowid() as i32)
    }

    pub fn get_by_id(tx: &Transaction, id: i32) -> SqliteResult<Option<ConfigModel>> {
        let mut stmt = tx.prepare("SELECT id, group_id, data, extra FROM configs WHERE id = ?1")?;

        let config = stmt
            .query_row(params![id], |row| {
                Ok(ConfigModel {
                    id: row.get(0)?,
                    group_id: row.get(1)?,
                    data: row.get(2)?,
                    extra: row.get(3)?,
                })
            })
            .optional()?;

        Ok(config)
    }

    pub fn get_by_ids(tx: &Transaction, ids: &[i32]) -> SqliteResult<Option<Vec<ConfigModel>>> {
        if ids.is_empty() {
            return Ok(Some(vec![]));
        }

        let vars = repeat_vars(ids.len());

        let sql = format!(
            "SELECT id, group_id, data, extra FROM configs WHERE id IN ({})",
            vars
        );

        let mut stmt = tx.prepare(&sql)?;

        let configs = stmt
            .query_map(rusqlite::params_from_iter(ids.iter().copied()), |row| {
                Ok(ConfigModel {
                    id: row.get(0)?,
                    group_id: row.get(1)?,
                    data: row.get(2)?,
                    extra: row.get(3)?,
                })
            })?
            .collect::<SqliteResult<Vec<ConfigModel>>>()?;

        Ok(Some(configs))
    }

    pub fn get_by_group_id(tx: &Transaction, group_id: i32) -> SqliteResult<Vec<ConfigModel>> {
        let mut stmt =
            tx.prepare("SELECT id, group_id, data, extra FROM configs WHERE group_id = ?1")?;

        let configs = stmt
            .query_map(params![group_id], |row| {
                Ok(ConfigModel {
                    id: row.get(0)?,
                    group_id: row.get(1)?,
                    data: row.get(2)?,
                    extra: row.get(3)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(configs)
    }
    pub fn get_by_group_id_with_pagination(
        tx: &Transaction,
        group_id: i32,
        pagination: PaginationParams,
    ) -> SqliteResult<Vec<ConfigModel>> {
        let mut stmt = tx.prepare(
            "SELECT id, group_id, data, extra FROM configs WHERE group_id = ?1 LIMIT ?2 OFFSET ?3",
        )?;

        let offset = (pagination.page as i64) * (pagination.limit as i64);

        let configs = stmt
            .query_map(params![group_id, pagination.limit, offset], |row| {
                Ok(ConfigModel {
                    id: row.get(0)?,
                    group_id: row.get(1)?,
                    data: row.get(2)?,
                    extra: row.get(3)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(configs)
    }

    pub fn get_all(tx: &Transaction) -> SqliteResult<Vec<ConfigModel>> {
        let mut stmt = tx.prepare("SELECT id, group_id, data, extra FROM configs")?;

        let configs = stmt
            .query_map([], |row| {
                Ok(ConfigModel {
                    id: row.get(0)?,
                    group_id: row.get(1)?,
                    data: row.get(2)?,
                    extra: row.get(3)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(configs)
    }

    pub fn update(tx: &Transaction, config: &ConfigModel) -> SqliteResult<()> {
        tx.execute(
            "UPDATE configs SET data = ?1, extra = ?2 WHERE id = ?3",
            params![&config.data, &config.extra, config.id],
        )?;

        Ok(())
    }

    pub fn delete(tx: &Transaction, id: i32) -> SqliteResult<bool> {
        let count = tx.execute("DELETE FROM configs WHERE id = ?1", params![id])?;
        Ok(count > 0)
    }

    pub fn delete_by_ids(tx: &Transaction, ids: &[i32]) -> SqliteResult<bool> {
        if ids.is_empty() {
            return Ok(false);
        }

        let vars = repeat_vars(ids.len());

        let sql = format!("DELETE FROM configs WHERE id = ({})", vars);

        let mut stmt = tx.prepare(&sql)?;
        let count = stmt.execute(params_from_iter(ids.iter().copied()))?;

        Ok(count > 0)
    }

    pub fn delete_by_group_id(tx: &Transaction, group_id: i32) -> SqliteResult<()> {
        tx.execute("DELETE FROM configs WHERE group_id = ?1", params![group_id])?;
        Ok(())
    }
}
