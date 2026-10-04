use std::fs;
use std::path::{Path, PathBuf};

use crate::constants::APP_NAME;
use crate::model::error::{SwarmError, SwarmResult};

const TEMPORARY_SUFFIX: &str = ".tmp";

pub fn application_directory() -> SwarmResult<PathBuf> {
    let directory = dirs::data_local_dir()
        .ok_or_else(|| SwarmError::Config("Unable to determine the data directory".to_owned()))?;

    Ok(directory.join(APP_NAME))
}

pub fn write_atomic(path: &Path, content: &[u8]) -> SwarmResult<()> {
    let directory = path.parent().ok_or_else(|| {
        SwarmError::Validation(format!("'{}' has no parent directory", path.display()))
    })?;

    fs::create_dir_all(directory)?;

    let mut temporary_name = path.as_os_str().to_os_string();

    temporary_name.push(TEMPORARY_SUFFIX);

    let temporary = PathBuf::from(temporary_name);

    fs::write(&temporary, content)?;
    fs::rename(&temporary, path)?;

    debug_assert!(path.is_file());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_atomic_write_replaces_the_content() {
        let directory =
            std::env::temp_dir().join(format!("swarm-storage-{}", uuid::Uuid::new_v4()));

        let path = directory.join("nested").join("file.json");

        write_atomic(&path, b"first").expect("the first write succeeds");
        write_atomic(&path, b"second").expect("the second write succeeds");

        let content = fs::read(&path).expect("the file is readable");

        fs::remove_dir_all(&directory).expect("the test directory is removable");

        assert_eq!(content, b"second");
    }
}
