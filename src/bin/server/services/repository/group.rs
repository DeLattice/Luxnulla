use rusqlite::{OptionalExtension, Result as SqliteResult, Transaction, params};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupModel {
    pub id: i32,
    pub name: String,
    pub subscribe_url: Option<Url>,
}

impl GroupModel {
    pub fn new(name: String, subscribe_url: Option<Url>) -> Self {
        Self {
            id: 0,
            name,
            subscribe_url,
        }
    }
}

pub struct GroupRepository;

impl GroupRepository {
    pub fn create(tx: &Transaction, group: &GroupModel) -> SqliteResult<i32> {
        let subscribe_url = group.subscribe_url.as_ref().map(|u| u.to_string());

        tx.execute(
            "INSERT INTO groups (name, subscribe_url) VALUES (?1, ?2)",
            params![&group.name, subscribe_url],
        )?;

        Ok(tx.last_insert_rowid() as i32)
    }

    pub fn get_by_id(tx: &Transaction, id: i32) -> SqliteResult<Option<GroupModel>> {
        let mut stmt = tx.prepare("SELECT id, name, subscribe_url FROM groups WHERE id = ?1")?;

        let group = stmt
            .query_row(params![id], |row| {
                let url_str: Option<String> = row.get(2)?;
                Ok(GroupModel {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    subscribe_url: url_str.and_then(|u| Url::parse(&u).ok()),
                })
            })
            .optional()?;

        Ok(group)
    }

    pub fn get_by_name(tx: &Transaction, name: &str) -> SqliteResult<Option<GroupModel>> {
        let mut stmt = tx.prepare("SELECT id, name, subscribe_url FROM groups WHERE name = ?1")?;

        let group = stmt
            .query_row(params![name], |row| {
                let url_str: Option<String> = row.get(2)?;
                Ok(GroupModel {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    subscribe_url: url_str.and_then(|u| Url::parse(&u).ok()),
                })
            })
            .optional()?;

        Ok(group)
    }

    pub fn get_all(tx: &Transaction) -> SqliteResult<Vec<GroupModel>> {
        let mut stmt = tx.prepare("SELECT id, name, subscribe_url FROM groups")?;

        let groups = stmt
            .query_map([], |row| {
                let url_str: Option<String> = row.get(2)?;
                Ok(GroupModel {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    subscribe_url: url_str.and_then(|u| Url::parse(&u).ok()),
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(groups)
    }

    pub fn update(tx: &Transaction, group: &GroupModel) -> SqliteResult<()> {
        let subscribe_url = group.subscribe_url.as_ref().map(|u| u.to_string());

        tx.execute(
            "UPDATE groups SET name = ?1, subscribe_url = ?2 WHERE id = ?3",
            params![&group.name, subscribe_url, group.id],
        )?;

        Ok(())
    }

    pub fn delete(tx: &Transaction, id: i32) -> SqliteResult<bool> {
        let rows_affected = tx.execute("DELETE FROM groups WHERE id = ?1", params![id])?;
        Ok(rows_affected > 0)
    }
}
