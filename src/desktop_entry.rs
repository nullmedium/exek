use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub categories: Vec<String>,
    pub desktop_file: PathBuf,
    pub terminal: bool,
}

impl AppEntry {
    pub fn from_path(path: &std::path::Path) -> Self {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Make the name more distinctive by adding [Path] prefix
        let display_name = format!("{} [Path]", name);

        Self {
            name: display_name,
            exec: path.display().to_string(),
            icon: None,
            comment: Some(path.display().to_string()),
            categories: vec!["Path".to_string()],
            desktop_file: PathBuf::new(),
            terminal: false,
        }
    }

    pub fn from_ini_file(path: PathBuf) -> Option<Self> {
        let content = std::fs::read_to_string(&path).ok()?;
        let mut name = None;
        let mut exec = None;
        let mut icon = None;
        let mut comment = None;
        let mut categories = Vec::new();
        let mut terminal = false;
        let mut no_display = false;
        let mut hidden = false;

        let mut in_desktop_entry = false;

        for line in content.lines() {
            let line = line.trim();

            if line == "[Desktop Entry]" {
                in_desktop_entry = true;
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry = false;
                continue;
            }

            if !in_desktop_entry || !line.contains('=') {
                continue;
            }

            let mut parts = line.splitn(2, '=');
            let key = parts.next()?.trim();
            let value = parts.next()?.trim();

            match key {
                "Name" => name = Some(value.to_string()),
                "Exec" => exec = Some(value.to_string()),
                "Icon" => icon = Some(value.to_string()),
                "Comment" => comment = Some(value.to_string()),
                "Categories" => {
                    categories = value.split(';')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                }
                "Terminal" => terminal = value.to_lowercase() == "true",
                "NoDisplay" => no_display = value.to_lowercase() == "true",
                "Hidden" => hidden = value.to_lowercase() == "true",
                _ => {}
            }
        }

        if no_display || hidden {
            return None;
        }

        Some(Self {
            name: name?,
            exec: exec?,
            icon,
            comment,
            categories,
            desktop_file: path,
            terminal,
        })
    }

    pub fn get_launch_command(&self) -> String {
        let exec = self.exec.clone();
        let exec = exec.replace("%F", "");
        let exec = exec.replace("%U", "");
        let exec = exec.replace("%f", "");
        let exec = exec.replace("%u", "");
        let exec = exec.replace("%i", "");
        let exec = exec.replace("%c", &self.name);
        let exec = exec.replace("%k", "");
        exec.trim().to_string()
    }
}

pub struct DesktopScanner;

impl DesktopScanner {
    pub fn scan() -> Result<Vec<AppEntry>> {
        let mut apps = HashMap::new();

        let desktop_dirs = vec![
            "/usr/share/applications",
            "/usr/local/share/applications",
            "~/.local/share/applications",
            "/var/lib/flatpak/exports/share/applications",
            "~/.local/share/flatpak/exports/share/applications",
        ];

        for dir in desktop_dirs {
            let expanded = shellexpand::tilde(dir);
            let path = std::path::Path::new(expanded.as_ref());

            if !path.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        if let Some(app) = AppEntry::from_ini_file(path) {
                            apps.insert(app.name.clone(), app);
                        }
                    }
                }
            }
        }

        Ok(apps.into_values().collect())
    }
}
