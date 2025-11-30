use rusqlite::{Connection, Result as SqliteResult};

use crate::services::repository::{
    config::{ConfigModel, ConfigRepository},
    group::{GroupModel, GroupRepository},
};

pub struct GroupConfigsTransactionService;

impl GroupConfigsTransactionService {
    pub fn create_group_with_configs(
        conn: &mut Connection,
        mut group: GroupModel,
        configs: &[ConfigModel],
    ) -> SqliteResult<(i32, Vec<i32>)> {
        let tx = conn.transaction()?;

        let group_id = GroupRepository::create(&tx, &group)?;
        group.id = group_id;

        // let mut config_ids = Vec::new();
        // for mut config in configs.iter() {
        //     config.group_id = group_id;
        //     let config_id = ConfigRepository::create(&tx, &config)?;
        //     config_ids.push(config_id);
        // }

        tx.commit()?;
        Ok((group_id, vec![1]))
    }

    pub fn update_group_and_configs(
        conn: &mut Connection,
        group: GroupModel,
        updated_configs: Vec<ConfigModel>,
    ) -> SqliteResult<()> {
        let tx = conn.transaction()?;

        GroupRepository::update(&tx, &group)?;

        for config in updated_configs {
            ConfigRepository::update(&tx, &config)?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn delete_group_with_configs(conn: &mut Connection, group_id: i32) -> SqliteResult<()> {
        let tx = conn.transaction()?;

        ConfigRepository::delete_by_group_id(&tx, group_id)?;
        GroupRepository::delete(&tx, group_id)?;

        tx.commit()?;
        Ok(())
    }

    pub fn move_configs_between_groups(
        conn: &mut Connection,
        from_group_id: i32,
        to_group_id: i32,
    ) -> SqliteResult<usize> {
        let tx = conn.transaction()?;

        let mut configs = ConfigRepository::get_by_group_id(&tx, from_group_id)?;

        for config in &mut configs {
            config.group_id = to_group_id;
            ConfigRepository::update(&tx, config)?;
        }

        tx.commit()?;
        Ok(configs.len())
    }

    pub fn add_configs_to_group(
        conn: &mut Connection,
        group_id: i32,
        configs: Vec<ConfigModel>,
    ) -> SqliteResult<Vec<i32>> {
        let tx = conn.transaction()?;

        GroupRepository::get_by_id(&tx, group_id)?
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;

        let mut config_ids = Vec::new();
        for mut config in configs {
            config.group_id = group_id;
            let config_id = ConfigRepository::create(&tx, &config)?;
            config_ids.push(config_id);
        }

        tx.commit()?;
        Ok(config_ids)
    }
}
