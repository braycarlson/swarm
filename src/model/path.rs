use std::path::{Path, PathBuf};

pub trait PathExtensions {
    fn clean_path(&self) -> PathBuf;
    fn file_name_string(&self) -> Option<String>;
    fn is_hidden(&self) -> bool;
    fn lowercase_name(&self) -> String;
}

impl PathExtensions for Path {
    fn clean_path(&self) -> PathBuf {
        dunce::canonicalize(self).unwrap_or_else(|_| self.to_path_buf())
    }

    fn file_name_string(&self) -> Option<String> {
        self.file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }

    fn is_hidden(&self) -> bool {
        self.file_name()
            .map(|name| name.to_string_lossy().starts_with('.'))
            .unwrap_or(false)
    }

    fn lowercase_name(&self) -> String {
        self.file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    }
}

impl PathExtensions for PathBuf {
    fn clean_path(&self) -> PathBuf {
        self.as_path().clean_path()
    }

    fn file_name_string(&self) -> Option<String> {
        self.as_path().file_name_string()
    }

    fn is_hidden(&self) -> bool {
        self.as_path().is_hidden()
    }

    fn lowercase_name(&self) -> String {
        self.as_path().lowercase_name()
    }
}
