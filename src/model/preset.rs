use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::constants::APP_NAME;
use crate::model::error::SwarmResult;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub root: PathBuf,
    pub paths: Vec<PathBuf>,
}

impl Preset {
    pub fn new(name: String, root: PathBuf, paths: Vec<PathBuf>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: now,
            root,
            paths,
        }
    }
}

#[derive(Clone)]
pub struct PresetModel {
    pub presets: HashMap<String, Preset>,
}

impl PresetModel {
    pub fn new() -> Self {
        Self {
            presets: HashMap::new(),
        }
    }

    pub fn add(&mut self, preset: Preset) {
        self.presets.insert(preset.id.clone(), preset);
    }

    pub fn remove(&mut self, id: &str) -> Option<Preset> {
        self.presets.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<&Preset> {
        self.presets.get(id)
    }

    pub fn list(&self) -> Vec<&Preset> {
        let mut presets: Vec<&Preset> = self.presets.values().collect();
        presets.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        presets
    }

    pub fn save_to_disk(&self) -> SwarmResult<()> {
        if let Some(dir) = Self::presets_directory() {
            fs::create_dir_all(&dir)?;

            for (id, preset) in &self.presets {
                let path = dir.join(format!("{}.json", id));
                let json = serde_json::to_string_pretty(preset)?;
                fs::write(path, json)?;
            }
        }

        Ok(())
    }

    pub fn delete_from_disk(id: &str) -> SwarmResult<()> {
        if let Some(dir) = Self::presets_directory() {
            let path = dir.join(format!("{}.json", id));
            let _ = fs::remove_file(path);
        }

        Ok(())
    }

    pub fn load_from_disk() -> Self {
        let mut model = Self::new();

        if let Some(dir) = Self::presets_directory() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.extension().and_then(|s| s.to_str()) == Some("json")
                        && let Ok(content) = fs::read_to_string(&path)
                            && let Ok(preset) = serde_json::from_str::<Preset>(&content) {
                                model.add(preset);
                            }
                }
            }
        }

        model
    }

    fn presets_directory() -> Option<PathBuf> {
        dirs::data_local_dir().map(|dir| {
            dir.join(APP_NAME.to_lowercase()).join("presets")
        })
    }
}

impl Default for PresetModel {
    fn default() -> Self {
        Self::new()
    }
}
