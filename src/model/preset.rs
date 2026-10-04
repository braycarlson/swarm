use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::constants::PRESET_COUNT_MAX;
use crate::model::error::{SwarmError, SwarmResult};
use crate::model::{identifier, storage, time};

const PRESET_EXTENSION: &str = "json";
const PRESETS_DIRECTORY_NAME: &str = "presets";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Preset {
    pub created_at: u64,
    pub identifier: String,
    pub name: String,
    pub paths: Option<Vec<PathBuf>>,
    pub query: Option<String>,
    pub root: Option<PathBuf>,
}

impl Preset {
    pub fn is_generic(&self) -> bool {
        self.root.is_none()
    }

    pub fn matches_root(&self, root: &Path) -> bool {
        self.root.as_deref() == Some(root)
    }

    pub fn new(name: String, selection: PresetSelection) -> Self {
        assert_ne!(name.trim(), "");

        Self {
            created_at: time::seconds_now(),
            identifier: identifier::generate(),
            name,
            paths: selection.paths,
            query: selection.query,
            root: selection.root,
        }
    }
}

#[derive(Default)]
pub struct PresetModel {
    presets: FxHashMap<String, Preset>,
}

impl PresetModel {
    fn path_for(directory: &Path, identifier: &str) -> PathBuf {
        assert!(identifier::is_valid(identifier));

        directory.join(format!("{identifier}.{PRESET_EXTENSION}"))
    }

    fn presets_directory() -> SwarmResult<PathBuf> {
        Ok(storage::application_directory()?.join(PRESETS_DIRECTORY_NAME))
    }

    fn read_preset(path: &Path) -> SwarmResult<Preset> {
        let content = fs::read_to_string(path)?;
        let preset: Preset = serde_json::from_str(&content)?;

        if !identifier::is_valid(&preset.identifier) {
            return Err(SwarmError::Validation(format!(
                "the identifier '{}' is not a valid file name",
                preset.identifier,
            )));
        }

        let stem = path.file_stem().and_then(|stem| stem.to_str());

        if stem != Some(preset.identifier.as_str()) {
            return Err(SwarmError::Validation(
                "the identifier does not match the file name".to_owned(),
            ));
        }

        Ok(preset)
    }

    pub fn add(&mut self, preset: Preset) {
        assert!(identifier::is_valid(&preset.identifier));

        let replaced = self.presets.insert(preset.identifier.clone(), preset);

        debug_assert!(replaced.is_none());
    }

    pub fn delete_from_disk(identifier: &str) -> SwarmResult<()> {
        if !identifier::is_valid(identifier) {
            return Err(SwarmError::Validation(format!(
                "the identifier '{identifier}' is not a valid file name",
            )));
        }

        let path = Self::path_for(&Self::presets_directory()?, identifier);

        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get(&self, identifier: &str) -> Option<&Preset> {
        self.presets.get(identifier)
    }

    pub fn is_empty(&self) -> bool {
        self.presets.is_empty()
    }

    pub fn list(&self) -> Vec<&Preset> {
        let mut presets: Vec<&Preset> = self.presets.values().collect();

        presets.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then_with(|| left.identifier.cmp(&right.identifier))
        });

        debug_assert_eq!(presets.len(), self.presets.len());

        presets
    }

    pub fn list_for_root(&self, root: Option<&Path>) -> Vec<&Preset> {
        let mut presets = self.list();

        presets.retain(|preset| {
            preset.is_generic() || root.is_some_and(|root_path| preset.matches_root(root_path))
        });

        presets
    }

    pub fn load_from_disk() -> Self {
        let mut model = Self::default();

        let Ok(directory) = Self::presets_directory() else {
            return model;
        };

        let Ok(entries) = fs::read_dir(&directory) else {
            return model;
        };

        let mut examined: u32 = 0;
        let mut skipped: u32 = 0;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.extension().and_then(|extension| extension.to_str()) != Some(PRESET_EXTENSION) {
                continue;
            }

            examined += 1;

            if examined > PRESET_COUNT_MAX {
                eprintln!(
                    "Stopped reading presets in {} at the limit of {PRESET_COUNT_MAX}",
                    directory.display(),
                );

                break;
            }

            match Self::read_preset(&path) {
                Ok(preset) => model.add(preset),
                Err(error) => {
                    eprintln!("Skipped the preset {}: {error}", path.display());

                    skipped += 1;
                }
            }
        }

        debug_assert!(skipped <= examined);

        model
    }

    pub fn remove(&mut self, identifier: &str) -> Option<Preset> {
        self.presets.remove(identifier)
    }

    pub fn save_to_disk(&self) -> SwarmResult<()> {
        let directory = Self::presets_directory()?;

        for preset in self.list() {
            let path = Self::path_for(&directory, &preset.identifier);
            let json = serde_json::to_string_pretty(preset)?;

            assert_ne!(json, "");

            storage::write_atomic(&path, json.as_bytes())?;
        }

        Ok(())
    }
}

pub struct PresetSelection {
    pub paths: Option<Vec<PathBuf>>,
    pub query: Option<String>,
    pub root: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection(root: Option<&str>) -> PresetSelection {
        PresetSelection {
            paths: None,
            query: None,
            root: root.map(PathBuf::from),
        }
    }

    #[test]
    fn a_generic_preset_matches_any_root() {
        let preset = Preset::new("generic".to_owned(), selection(None));

        assert!(preset.is_generic());
        assert!(!preset.matches_root(Path::new("/a")));
    }

    #[test]
    fn a_rooted_preset_matches_its_own_root_only() {
        let preset = Preset::new("rooted".to_owned(), selection(Some("/a")));

        assert!(preset.matches_root(Path::new("/a")));
        assert!(!preset.matches_root(Path::new("/b")));
    }

    #[test]
    fn listing_for_a_root_keeps_generic_presets() {
        let mut model = PresetModel::default();

        model.add(Preset::new("generic".to_owned(), selection(None)));
        model.add(Preset::new("rooted".to_owned(), selection(Some("/a"))));
        model.add(Preset::new("other".to_owned(), selection(Some("/b"))));

        assert_eq!(model.list_for_root(Some(Path::new("/a"))).len(), 2);
        assert_eq!(model.list_for_root(None).len(), 1);
    }

    #[test]
    fn a_preset_whose_identifier_escapes_the_directory_is_rejected() {
        let directory = std::env::temp_dir().join(format!("swarm-preset-{}", uuid::Uuid::new_v4()));

        fs::create_dir_all(&directory).expect("the test directory is creatable");

        let mut preset = Preset::new("escape".to_owned(), selection(None));

        preset.identifier = "../../escape".to_owned();

        let path = directory.join("escape.json");
        let json = serde_json::to_string(&preset).expect("the preset serializes");

        fs::write(&path, json).expect("the preset file is writable");

        let outcome = PresetModel::read_preset(&path);

        fs::remove_dir_all(&directory).expect("the test directory is removable");

        assert!(outcome.is_err());
    }

    #[test]
    fn a_preset_whose_identifier_differs_from_its_file_name_is_rejected() {
        let directory = std::env::temp_dir().join(format!("swarm-preset-{}", uuid::Uuid::new_v4()));

        fs::create_dir_all(&directory).expect("the test directory is creatable");

        let preset = Preset::new("renamed".to_owned(), selection(None));
        let path = directory.join("other.json");
        let json = serde_json::to_string(&preset).expect("the preset serializes");

        fs::write(&path, json).expect("the preset file is writable");

        let outcome = PresetModel::read_preset(&path);

        fs::remove_dir_all(&directory).expect("the test directory is removable");

        assert!(outcome.is_err());
    }
}
