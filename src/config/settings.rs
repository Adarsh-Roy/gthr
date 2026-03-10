use crate::constants::{DEFAULT_MAX_CLIPBOARD_SIZE, DEFAULT_MAX_FILE_SIZE};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,
    #[serde(default = "default_max_clipboard_size")]
    pub max_clipboard_size: usize,
    #[serde(default = "default_respect_gitignore")]
    pub respect_gitignore: bool,
    #[serde(default = "default_show_hidden")]
    pub show_hidden: bool,
    #[serde(default)]
    pub extra_text_extensions: Vec<String>,
    #[serde(default)]
    pub exclude_text_extensions: Vec<String>,
}

fn default_max_file_size() -> u64 {
    DEFAULT_MAX_FILE_SIZE
}
fn default_max_clipboard_size() -> usize {
    DEFAULT_MAX_CLIPBOARD_SIZE
}
fn default_respect_gitignore() -> bool {
    true
}
fn default_show_hidden() -> bool {
    false
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_file_size: default_max_file_size(),
            max_clipboard_size: default_max_clipboard_size(),
            respect_gitignore: default_respect_gitignore(),
            show_hidden: default_show_hidden(),
            extra_text_extensions: Vec::new(),
            exclude_text_extensions: Vec::new(),
        }
    }
}

impl Settings {
    pub fn load_from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let settings: Settings = toml::from_str(&content)?;
        Ok(settings)
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_global_config_path() -> std::path::PathBuf {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return std::path::PathBuf::from(xdg).join("gthr.toml");
        }
        if let Some(home) = dirs::home_dir() {
            return home.join(".config").join("gthr.toml");
        }
        std::path::PathBuf::from("gthr.toml")
    }

    /// Load settings from ~/.config/gthr.toml, creating it with defaults if absent.
    pub fn load() -> Self {
        let path = Self::get_global_config_path();
        if !path.exists() {
            let _ = Self::create_default_config(&path);
        }
        Self::load_from_file(&path).unwrap_or_default()
    }

    fn create_default_config(path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = r#"# gthr configuration
# This file is auto-generated. All values shown are defaults.
#
# Priority order:
#   1. CLI flags passed when invoking gthr
#   2. This config file (lowest)

# Maximum file size to include (in bytes)
# Files larger than this are skipped
# max_file_size = 2097152  # 2MB

# Maximum content size for clipboard copy (in bytes)
# Content larger than this triggers a file save prompt
# max_clipboard_size = 2097152  # 2MB

# Whether to respect .gitignore rules when traversing directories
# respect_gitignore = true

# Whether to show hidden files and directories (dotfiles)
# show_hidden = false

# Additional file extensions to always treat as text files
# These are added on top of the built-in detection (~60 extensions + content heuristics)
# Example: extra_text_extensions = ["mdx", "astro", "prisma"]
# extra_text_extensions = []

# File extensions to never treat as text files
# Overrides both the built-in list and content-based detection
# Takes priority over extra_text_extensions
# Example: exclude_text_extensions = ["log", "sql", "min.js"]
# exclude_text_extensions = []
"#;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn text_extension_overrides(&self) -> (HashSet<String>, HashSet<String>) {
        let extra = self
            .extra_text_extensions
            .iter()
            .map(|s| s.to_lowercase().trim_start_matches('.').to_string())
            .collect();
        let exclude = self
            .exclude_text_extensions
            .iter()
            .map(|s| s.to_lowercase().trim_start_matches('.').to_string())
            .collect();
        (extra, exclude)
    }

    pub fn format_clipboard_size(&self) -> String {
        let size = self.max_clipboard_size;
        if size >= 1024 * 1024 {
            format!("{}MB", size / (1024 * 1024))
        } else if size >= 1024 {
            format!("{}KB", size / 1024)
        } else {
            format!("{}B", size)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_settings_serialization() -> Result<()> {
        let settings = Settings::default();
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.toml");

        settings.save_to_file(&config_path)?;
        let loaded_settings = Settings::load_from_file(&config_path)?;

        assert_eq!(settings.max_file_size, loaded_settings.max_file_size);
        assert_eq!(settings.respect_gitignore, loaded_settings.respect_gitignore);

        Ok(())
    }

    #[test]
    fn test_settings_with_text_extensions_roundtrip() -> Result<()> {
        let mut settings = Settings::default();
        settings.extra_text_extensions = vec!["mdx".to_string(), "astro".to_string()];
        settings.exclude_text_extensions = vec!["log".to_string(), "sql".to_string()];

        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.toml");

        settings.save_to_file(&config_path)?;
        let loaded = Settings::load_from_file(&config_path)?;

        assert_eq!(loaded.extra_text_extensions, vec!["mdx", "astro"]);
        assert_eq!(loaded.exclude_text_extensions, vec!["log", "sql"]);

        Ok(())
    }

    #[test]
    fn test_text_extension_overrides_normalizes() {
        let mut settings = Settings::default();
        settings.extra_text_extensions = vec![".RS".to_string(), ".prisma".to_string()];
        settings.exclude_text_extensions = vec![".LOG".to_string(), "SQL".to_string()];

        let (extra, exclude) = settings.text_extension_overrides();

        assert!(extra.contains("rs"));
        assert!(extra.contains("prisma"));
        assert!(exclude.contains("log"));
        assert!(exclude.contains("sql"));
    }

    #[test]
    fn test_settings_without_text_extensions_defaults_empty() -> Result<()> {
        let toml_content = r#"
max_file_size = 1048576
respect_gitignore = true
"#;
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, toml_content)?;

        let loaded = Settings::load_from_file(&config_path)?;
        assert!(loaded.extra_text_extensions.is_empty());
        assert!(loaded.exclude_text_extensions.is_empty());

        Ok(())
    }
}
