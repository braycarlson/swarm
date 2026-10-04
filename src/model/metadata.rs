use std::fs::{self, File};
use std::io::{ErrorKind, Read as _};
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::constants::LINE_COUNT_BYTES_MAX;

const LINE_BUFFER_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileMetadata {
    pub lines: Option<u64>,
    pub modified: Option<u64>,
    pub size: u64,
}

impl FileMetadata {
    pub fn from_path(path: &Path, lines_needed: bool) -> Option<Self> {
        let metadata = fs::metadata(path).ok()?;

        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());

        let lines = if lines_needed {
            line_count(path, &metadata)
        } else {
            None
        };

        Some(Self {
            lines,
            modified,
            size: metadata.len(),
        })
    }
}

#[inline(never)]
fn line_count(path: &Path, metadata: &fs::Metadata) -> Option<u64> {
    if !metadata.is_file() {
        return None;
    }

    let mut file = File::open(path).ok()?;
    let mut buffer = [0_u8; LINE_BUFFER_BYTES];
    let mut byte_last = b'\n';
    let mut consumed_bytes_total: u64 = 0;
    let mut count: u64 = 0;

    while consumed_bytes_total <= LINE_COUNT_BYTES_MAX {
        let read = match file.read(&mut buffer) {
            Ok(read) => read,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(_) => return None,
        };

        if read == 0 {
            if byte_last != b'\n' {
                count += 1;
            }

            debug_assert!(count == 0 || consumed_bytes_total > 0);

            return Some(count);
        }

        assert!(read <= buffer.len());

        let chunk = &buffer[..read];

        count += memchr::memchr_iter(b'\n', chunk).count() as u64;
        byte_last = chunk[read - 1];
        consumed_bytes_total += read as u64;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_of(content: &str) -> Option<u64> {
        let path = std::env::temp_dir().join(format!("swarm-lines-{}", uuid::Uuid::new_v4()));

        fs::write(&path, content).expect("the test file is writable");

        let metadata = fs::metadata(&path).expect("the test file has metadata");
        let count = line_count(&path, &metadata);

        fs::remove_file(&path).expect("the test file is removable");

        count
    }

    #[test]
    fn a_trailing_line_without_a_newline_counts() {
        assert_eq!(count_of(""), Some(0));
        assert_eq!(count_of("a"), Some(1));
        assert_eq!(count_of("a\n"), Some(1));
        assert_eq!(count_of("a\nb"), Some(2));
        assert_eq!(count_of("a\nb\n"), Some(2));
    }

    #[test]
    fn a_missing_file_has_no_metadata() {
        assert_eq!(FileMetadata::from_path(Path::new("/nonexistent/swarm"), true), None);
    }

    #[test]
    fn lines_are_only_counted_on_request() {
        let path = std::env::temp_dir().join(format!("swarm-metadata-{}", uuid::Uuid::new_v4()));

        fs::write(&path, "one\ntwo\n").expect("the test file is writable");

        let basic = FileMetadata::from_path(&path, false).expect("the file has metadata");
        let counted = FileMetadata::from_path(&path, true).expect("the file has metadata");

        fs::remove_file(&path).expect("the test file is removable");

        assert_eq!(basic.lines, None);
        assert_eq!(counted.lines, Some(2));
        assert_eq!(counted.size, 8);
    }
}
