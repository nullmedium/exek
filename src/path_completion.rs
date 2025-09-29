use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PathCompletion {
    pub path: PathBuf,
    pub display_name: String,
    pub is_dir: bool,
}

pub struct PathCompleter;

impl PathCompleter {
    pub fn new() -> Self {
        Self
    }

    pub fn is_path_query(query: &str) -> bool {
        query.starts_with('/')
            || query.starts_with("./")
            || query.starts_with("../")
            || query.starts_with('~')
    }

    pub fn complete_path(&self, query: &str) -> Vec<PathCompletion> {
        if !Self::is_path_query(query) {
            return Vec::new();
        }

        let expanded = shellexpand::tilde(query);
        let path_str = expanded.as_ref();

        let (dir_path, prefix) = if path_str.ends_with('/') {
            (path_str.to_string(), String::new())
        } else {
            let path = Path::new(path_str);
            if let Some(parent) = path.parent() {
                let parent_str = if parent.as_os_str().is_empty() {
                    ".".to_string()
                } else {
                    parent.to_string_lossy().to_string()
                };
                let file_name = path.file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                (parent_str, file_name)
            } else {
                (".".to_string(), path_str.to_string())
            }
        };

        let mut completions = Vec::new();

        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let file_name = entry.file_name().to_string_lossy().to_string();

                    if !file_name.starts_with(&prefix) {
                        continue;
                    }

                    let is_dir = metadata.is_dir();
                    let is_executable = self.is_executable(&entry.path());

                    if is_dir || is_executable {
                        let full_path = entry.path();
                        let display_name = if query.starts_with('~') {
                            let home = dirs::home_dir().unwrap_or_default();
                            if let Ok(relative) = full_path.strip_prefix(&home) {
                                format!("~/{}", relative.display())
                            } else {
                                full_path.to_string_lossy().to_string()
                            }
                        } else {
                            full_path.to_string_lossy().to_string()
                        };

                        completions.push(PathCompletion {
                            path: full_path,
                            display_name,
                            is_dir,
                        });
                    }
                }
            }
        }

        completions.sort_by(|a, b| {
            a.is_dir.cmp(&b.is_dir).reverse()
                .then_with(|| a.display_name.cmp(&b.display_name))
        });

        completions
    }

    fn is_executable(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        if let Ok(metadata) = fs::metadata(path) {
            let permissions = metadata.permissions();
            let mode = permissions.mode();

            let user_exec = mode & 0o100 != 0;
            let group_exec = mode & 0o010 != 0;
            let other_exec = mode & 0o001 != 0;

            user_exec || group_exec || other_exec
        } else {
            false
        }
    }

    pub fn apply_completion(_query: &str, completion: &PathCompletion) -> String {
        let mut result = completion.display_name.clone();

        if completion.is_dir && !result.ends_with('/') {
            result.push('/');
        }

        result
    }
}
