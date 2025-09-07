use eyre::OptionExt;
use luxnulla::CONFIG_DIR;
use notify::{EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub configs: JsonValue,
}

impl Group {
    pub fn new(name: String, configs: JsonValue) -> Self {
        Self { name, configs }
    }
}

#[derive(Debug, Clone)]
pub struct StorageService {
    groups: Arc<RwLock<HashMap<String, Group>>>,
    groups_dir: Option<PathBuf>,
}

impl StorageService {
    pub fn new() -> Self {
        let config_dir_path = dirs::config_dir().unwrap().join(CONFIG_DIR);
        let groups_dir = config_dir_path.join("groups");

        if !groups_dir.exists() {
            fs::create_dir_all(&groups_dir)
                .unwrap_or_else(|e| panic!("Failed to create groups directory: {}", e));
        }

        let instance = Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            groups_dir: Some(groups_dir),
        };

        if let Err(e) = instance.load_groups_from_disk() {
            eprintln!("Warning: Could not load groups from disk: {}", e);
        }

        let watchable_instance = Arc::new(instance.clone());
        watchable_instance.watch_dog();

        instance
    }

    fn load_groups_from_disk(&self) -> Result<(), StorageError> {
        if let Some(groups_dir) = &self.groups_dir {
            let mut groups = self.groups.write().map_err(|_| StorageError::LockError)?;

            for entry in
                fs::read_dir(groups_dir).map_err(|e| StorageError::FileError(e.to_string()))?
            {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("Warning: Failed to read directory entry: {}", e);
                        continue;
                    }
                };

                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    match fs::read_to_string(&path) {
                        Ok(content) => match serde_json::from_str::<Group>(&content) {
                            Ok(group) => {
                                groups.insert(group.name.clone(), group);
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to parse group file at {:?}: {}",
                                    path, e
                                );
                            }
                        },
                        Err(e) => {
                            eprintln!("Warning: Failed to read group file at {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn watch_dog(self: Arc<Self>) {
        tokio::spawn(async move {
            println!("Наблюдение за директорией...");
            if let Some(ref path) = self.groups_dir {
                // We're now calling a method on a cloned Arc
                if let Err(e) = self.watch_directory(path.clone(), self.groups.clone()) {
                    eprintln!("Ошибка наблюдения: {:?}", e);
                }
            }
        });
    }

    fn watch_directory(
        &self,
        path: PathBuf,
        groups_arc: Arc<RwLock<HashMap<String, Group>>>,
    ) -> Result<(), anyhow::Error> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::RecommendedWatcher::new(tx, notify::Config::default())?;
        watcher.watch(&path, RecursiveMode::Recursive)?;

        for res in rx {
            let event = res?;
            let event_path = event
                .paths
                .first()
                .ok_or_eyre("No path found in event")
                .unwrap();

            match event.kind {
                EventKind::Remove(_) => {
                    if event_path == &path {
                        println!("The watched directory was removed. Clearing groups.");
                        let mut groups = groups_arc.write().unwrap();
                        groups.clear();
                        return Ok(());
                    } else if event_path.is_file() {
                        if let Some(file_name) = event_path.file_stem().and_then(|s| s.to_str()) {
                            let mut groups = groups_arc.write().unwrap();
                            if groups.remove(file_name).is_some() {
                                println!("Group '{}' was removed from memory.", file_name);
                            }
                        }
                    }
                }
                EventKind::Modify(_) => {
                    if event_path.is_file() {
                        println!("File updated: {:?}", event_path);
                        if let Ok(content) = fs::read_to_string(event_path) {
                            if let Ok(group) = serde_json::from_str::<Group>(&content) {
                                let mut groups = groups_arc.write().unwrap();
                                groups.insert(group.name.clone(), group);
                                println!("Group updated in memory.");
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn store_group(&self, group: Group) -> Result<(), StorageError> {
        let group_name = group.name.clone();
        {
            let mut groups = self.groups.write().map_err(|_| StorageError::LockError)?;
            groups.insert(group.name.clone(), group);
        }
        self.save_group_to_file(&group_name)?;
        Ok(())
    }

    pub fn get_group(&self, name: &str) -> Result<Option<Group>, StorageError> {
        let groups = self.groups.read().map_err(|_| StorageError::LockError)?;
        Ok(groups.get(name).cloned())
    }

    pub fn get_all_groups(&self) -> Result<Vec<Group>, StorageError> {
        let groups = self.groups.read().map_err(|_| StorageError::LockError)?;

        Ok(groups.values().cloned().collect())
    }

    pub fn update_group_config(&self, group: Group) -> Result<bool, StorageError> {
        let mut is_group_updated = false;
        let group_name = group.name.clone();

        {
            let mut groups = self.groups.write().map_err(|_| StorageError::LockError)?;

            match groups.get(&group_name) {
                Some(existing_group) => {
                    let updated_group = Group {
                        name: existing_group.name.clone(),
                        configs: group.configs.clone(),
                    };

                    groups.insert(updated_group.name.clone(), updated_group);
                    is_group_updated = true;

                    group.configs.clone()
                }
                None => return Err(StorageError::GroupNotFound(group_name.to_string())),
            }
        };

        if is_group_updated {
            self.save_group_to_file(&group_name)?;
        }

        Ok(true)
    }

    pub fn delete_group(&self, name: &str) -> Result<bool, StorageError> {
        let result = self
            .groups
            .write()
            .map_err(|_| StorageError::LockError)?
            .remove(name)
            .is_some();

        if result {
            self.delete_group_file(name)?;
        }
        Ok(result)
    }

    pub fn delete_all_group(&self) -> Result<(), StorageError> {
        let group_names: Vec<String> = {
            let groups_read = self.groups.read().map_err(|_| StorageError::LockError)?;
            groups_read.keys().cloned().collect()
        };

        for name in group_names {
            self.delete_group_file(&name)?;
        }

        {
            self.groups.write().unwrap().clear();
        }

        Ok(())
    }

    pub fn delete_group_file(&self, group_name: &str) -> Result<(), StorageError> {
        if let Some(ref groups_dir) = self.groups_dir {
            let file_path = Path::new(groups_dir).join(format!("{}.json", group_name));
            if file_path.exists() {
                fs::remove_file(file_path).map_err(|e| StorageError::FileError(e.to_string()))?;
            }
        }
        Ok(())
    }

    pub fn group_exists(&self, name: &str) -> Result<bool, StorageError> {
        let groups = self.groups.read().map_err(|_| StorageError::LockError)?;
        Ok(groups.contains_key(name))
    }

    pub fn count_groups(&self) -> Result<usize, StorageError> {
        let groups = self.groups.read().map_err(|_| StorageError::LockError)?;
        Ok(groups.len())
    }

    pub fn list_group_names(&self) -> Result<Vec<String>, StorageError> {
        let groups = self.groups.read().map_err(|_| StorageError::LockError)?;
        Ok(groups.keys().cloned().collect())
    }

    pub fn upsert_group(&self, group: Group) -> Result<bool, StorageError> {
        let group_name = group.name.clone();
        let existed = {
            let mut groups = self.groups.write().map_err(|_| StorageError::LockError)?;
            let existed = groups.contains_key(&group.name);
            groups.insert(group.name.clone(), group);
            existed
        };
        self.save_group_to_file(&group_name)?;
        Ok(existed)
    }

    pub fn save_group_to_file(&self, group_name: &str) -> Result<(), StorageError> {
        let groups = self.groups.read().map_err(|_| StorageError::LockError)?;

        match groups.get(group_name) {
            Some(group) => {
                let groups_dir = match &self.groups_dir {
                    Some(val) => val.clone(),
                    None => PathBuf::new(),
                };

                let file_path = groups_dir.join(format!("{}.json", group_name));

                let json_data = serde_json::to_string_pretty(group)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                fs::write(file_path, json_data)
                    .map_err(|e| StorageError::FileError(e.to_string()))?;

                Ok(())
            }
            None => Err(StorageError::GroupNotFound(group_name.to_string())),
        }
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

    #[error("Invalid JSON configuration: {0}")]
    InvalidJson(String),

    #[error("Storage operation failed: {0}")]
    OperationFailed(String),

    #[error("File operation failed: {0}")]
    FileError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}
