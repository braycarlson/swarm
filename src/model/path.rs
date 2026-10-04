use std::path::{Path, PathBuf};

pub fn canonical(path: &Path) -> PathBuf {
    dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub fn directory_of(path: &Path) -> PathBuf {
    if !path.is_file() {
        return path.to_path_buf();
    }

    path.parent()
        .map_or_else(|| path.to_path_buf(), Path::to_path_buf)
}

pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.as_encoded_bytes().first() == Some(&b'.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_is_its_own_directory() {
        let directory = std::env::temp_dir();

        assert_eq!(directory_of(&directory), directory);
    }

    #[test]
    fn a_missing_path_passes_through() {
        let missing = Path::new("/nonexistent/swarm/path");

        assert_eq!(directory_of(missing), missing);
    }

    #[test]
    fn dotfiles_are_hidden() {
        assert!(is_hidden(Path::new("/a/.git")));
        assert!(!is_hidden(Path::new("/a/src")));
        assert!(!is_hidden(Path::new("/")));
    }
}
