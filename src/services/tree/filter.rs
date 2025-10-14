use std::path::Path;

use crate::model::options::Options;
use crate::model::path::PathExtensions;

pub fn is_binary_file(path: &Path) -> bool {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    if let Some(ext) = extension {
        matches!(
            ext.as_str(),
            // Image files
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "ico" | "webp" |
            "tiff" | "tif" | "psd" | "raw" | "heif" | "heic" | "indd" | "ai" |
            "eps" | "pdf" | "cr2" | "nef" | "orf" | "sr2" | "dng" |

            // Video files
            "mp4" | "avi" | "mov" | "wmv" | "flv" | "mkv" | "webm" | "m4v" |
            "mpg" | "mpeg" | "3gp" | "ogv" | "m2ts" | "mts" | "vob" | "ts" |

            // Audio files
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" |
            "ape" | "alac" | "aiff" | "au" | "mid" | "midi" | "ra" | "rm" |

            // Archive files
            "zip" | "tar" | "gz" | "rar" | "7z" | "bz2" | "xz" | "tgz" |
            "tbz2" | "lz" | "lzma" | "z" | "cab" | "iso" | "dmg" | "pkg" |
            "deb" | "rpm" | "apk" | "msi" |

            // Executable and binary files
            "exe" | "dll" | "so" | "dylib" | "lib" | "a" | "o" | "obj" |
            "pdb" | "class" | "jar" | "war" | "ear" | "bin" | "dat" |
            "app" | "com" | "bat" | "cmd" | "sys" | "drv" | "res" |

            // Database files
            "db" | "sqlite" | "sqlite3" | "mdb" | "accdb" | "dbf" | "sdf" |
            "bak" | "db3" | "fdb" | "gdb" | "kdb" |

            // Font files
            "ttf" | "otf" | "woff" | "woff2" | "eot" | "fnt" | "fon" |
            "pfb" | "pfm" |

            // Document files
            "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" |
            "ods" | "odp" | "pages" | "numbers" | "key" | "rtf" |

            // Compiled and intermediate files
            "pyc" | "pyo" | "pyd" | "elc" | "rbc" | "beam" | "fasl" |

            // 3D model files
            "fbx" | "dae" | "3ds" | "blend" | "c4d" | "max" |
            "ma" | "mb" | "stl" | "ply" |

            // Game assets
            "unity3d" | "unitypackage" | "asset" | "prefab" | "pak" |
            "vpk" | "wad" | "bsp" |

            // Virtual machine and disk images
            "vdi" | "vmdk" | "vhd" | "vhdx" | "qcow2" | "img" | "toast" |

            // Backup and temporary files
            "lock" | "tmp" | "temp" | "swp" | "swo" | "old" | "orig" | "cache" |

            // Encrypted and protected files
            "env" | "enc" | "gpg" | "aes" | "pgp" | "p12" | "pfx" | "keystore" |

            // Other binary formats
            "crx" | "xpi" | "safariextz" | "ipa" | "aab" |
            "nupkg" | "snupkg" | "vsix" | "gem" | "whl" | "egg"
        )
    } else {
        false
    }
}

pub fn is_development_directory(path: &Path) -> bool {
    if let Some(dir_name) = path.file_name_string() {
        matches!(
            dir_name.as_str(),
            // Version control
            ".git" | ".svn" | ".hg" | ".bzr" | ".fossil" | "_darcs" |

            // Build outputs
            "target" | "build" | "dist" | "out" | "bin" | "obj" |
            "_build" | ".build" | "release" | "debug" | "Release" | "Debug" |

            // Dependencies
            "node_modules" | "bower_components" | "jspm_packages" | "vendor" |
            "packages" | ".bundle" | "deps" | "_deps" |

            // Python
            "__pycache__" | ".pytest_cache" | ".mypy_cache" | ".ruff_cache" |
            ".pytype" | ".tox" | "venv" | ".venv" | "env" | ".env" |
            "virtualenv" | ".virtualenv" | "ENV" | ".eggs" | "*.egg-info" |
            ".Python" | "pip-log.txt" | "pip-delete-this-directory.txt" |

            // JavaScript/TypeScript
            ".npm" | ".yarn" | ".pnp" | ".pnp.js" | ".next" | ".nuxt" |
            ".cache" | ".parcel-cache" | ".turbo" | ".vercel" | ".docusaurus" |

            // Java/JVM
            ".gradle" | ".mvn" | ".m2" | ".settings" |

            // .NET
            ".vs" | ".vscode" |

            // Ruby
            ".gem" | "tmp" |

            // Go
            "Godeps" |

            // PHP
            "composer.lock" |

            // IDEs and Editors
            ".idea" | ".vscode-test" | ".fleet" |
            ".eclipse" | ".project" | ".classpath" |
            ".sublime-project" | ".sublime-workspace" | "*.swp" | "*.swo" |

            // Testing and Coverage
            "coverage" | ".coverage" | "htmlcov" | ".nyc_output" |
            "test-results" | "test-reports" | ".jest" |

            // Documentation
            "site" | "docs/_build" | "_site" |

            // Logs and temporary files
            "logs" | "log" | "*.log" | "temp" | ".tmp" | ".temp" |

            // OS-specific
            ".DS_Store" | "Thumbs.db" | "Desktop.ini" | "$RECYCLE.BIN" |

            // Other common development artifacts
            ".terraform" | ".vagrant" | ".docker" | ".devcontainer" |
            ".history" | ".metals" | ".bloop" |
            "CMakeFiles" | "cmake-build-debug" | "cmake-build-release"
        )
    } else {
        false
    }
}

pub fn is_path_in_excluded_patterns(path: &Path, exclude_patterns: &[String]) -> bool {
    if let Some(filename) = path.file_name_string() {
        if exclude_patterns.iter()
            .any(|pattern| !pattern.is_empty() && pattern_matches(&filename, pattern))
        {
            return true;
        }
    }

    let mut current = path;

    while let Some(parent) = current.parent() {
        if let Some(parent_name) = parent.file_name_string() {
            if exclude_patterns.iter()
                .any(|pattern| !pattern.is_empty() && pattern_matches(&parent_name, pattern))
            {
                return true;
            }
        }
        current = parent;
    }

    false
}

pub fn pattern_matches(filename: &str, pattern: &str) -> bool {
    let pattern = pattern.trim();

    if pattern == "*" {
        return true;
    }

    if !pattern.contains('*') {
        return filename == pattern;
    }

    match (pattern.starts_with('*'), pattern.ends_with('*')) {
        (true, true) => {
            let middle = &pattern[1..pattern.len() - 1];
            filename.contains(middle)
        }
        (true, false) => {
            let suffix = &pattern[1..];
            filename.ends_with(suffix)
        }
        (false, true) => {
            let prefix = &pattern[..pattern.len() - 1];
            filename.starts_with(prefix)
        }
        (false, false) => {
            if pattern.starts_with("*.") {
                let extension = &pattern[1..];
                filename.ends_with(extension)
            } else {
                false
            }
        }
    }
}

pub fn should_include_path(path: &Path, options: &Options) -> bool {
    if path.is_dir() && is_development_directory(path) {
        return false;
    }

    if path.is_dir() {
        return !is_path_in_excluded_patterns(path, &options.exclude);
    }

    if is_binary_file(path) {
        return false;
    }

    if !options.include.is_empty() {
        let include_matched = options.include.iter()
            .any(|pattern| {
                !pattern.is_empty()
                    && path.file_name_string()
                        .map_or(false, |name| pattern_matches(&name, pattern))
            });

        if !include_matched {
            return false;
        }

        if is_path_in_excluded_patterns(path, &options.exclude) {
            return false;
        }

        return true;
    }

    !is_path_in_excluded_patterns(path, &options.exclude)
}
