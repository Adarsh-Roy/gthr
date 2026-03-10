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
    #[serde(default)]
    pub include_patterns: Vec<String>,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    #[serde(default)]
    pub hide_patterns: Vec<String>,
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
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            hide_patterns: Vec::new(),
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

    pub fn get_local_config_path(root: &std::path::Path) -> std::path::PathBuf {
        root.join("gthr.toml")
    }

    /// Load settings by merging global config, local config (in root dir), with local taking priority.
    pub fn load_with_local(root: &std::path::Path) -> Self {
        let global_path = Self::get_global_config_path();
        if !global_path.exists() {
            let _ = Self::create_default_config(&global_path);
        }
        let global = PartialSettings::load_from_file(&global_path);

        let local_path = Self::get_local_config_path(root);
        let local = PartialSettings::load_from_file(&local_path);

        global.merge(local).resolve()
    }

    fn create_default_config(path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = r#"# gthr configuration
# This file is auto-generated. All values shown are defaults.
#
# Priority order:
#   1. CLI flags (highest)
#   2. Local gthr.toml in project root directory
#   3. This global config file
#   4. Built-in defaults (lowest)

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

# Glob patterns to include files (applied in both interactive and direct modes)
# Example: include_patterns = ["src/**/*.rs", "*.toml"]
# include_patterns = []

# Glob patterns to exclude files (applied in both interactive and direct modes)
# Example: exclude_patterns = ["target/**", "*.log"]
# exclude_patterns = []

# Glob patterns to hide from the tree (excluded files still appear; hidden files don't)
# Stacks with show_hidden: hides additional files, does not reveal hidden ones
# Example: hide_patterns = ["target/**", "*.log", "node_modules"]
# hide_patterns = []
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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PartialSettings {
    pub max_file_size: Option<u64>,
    pub max_clipboard_size: Option<usize>,
    pub respect_gitignore: Option<bool>,
    pub show_hidden: Option<bool>,
    pub extra_text_extensions: Option<Vec<String>>,
    pub exclude_text_extensions: Option<Vec<String>>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub hide_patterns: Option<Vec<String>>,
}

impl PartialSettings {
    pub fn load_from_file(path: &std::path::Path) -> Self {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        toml::from_str(&content).unwrap_or_default()
    }

    /// Merge two PartialSettings. `other` takes priority (local overrides global).
    pub fn merge(self, other: PartialSettings) -> PartialSettings {
        PartialSettings {
            max_file_size: other.max_file_size.or(self.max_file_size),
            max_clipboard_size: other.max_clipboard_size.or(self.max_clipboard_size),
            respect_gitignore: other.respect_gitignore.or(self.respect_gitignore),
            show_hidden: other.show_hidden.or(self.show_hidden),
            extra_text_extensions: other.extra_text_extensions.or(self.extra_text_extensions),
            exclude_text_extensions: other.exclude_text_extensions.or(self.exclude_text_extensions),
            include_patterns: other.include_patterns.or(self.include_patterns),
            exclude_patterns: other.exclude_patterns.or(self.exclude_patterns),
            hide_patterns: other.hide_patterns.or(self.hide_patterns),
        }
    }

    /// Resolve into a full Settings, filling None fields with defaults.
    pub fn resolve(self) -> Settings {
        let defaults = Settings::default();
        Settings {
            max_file_size: self.max_file_size.unwrap_or(defaults.max_file_size),
            max_clipboard_size: self.max_clipboard_size.unwrap_or(defaults.max_clipboard_size),
            respect_gitignore: self.respect_gitignore.unwrap_or(defaults.respect_gitignore),
            show_hidden: self.show_hidden.unwrap_or(defaults.show_hidden),
            extra_text_extensions: self.extra_text_extensions.unwrap_or(defaults.extra_text_extensions),
            exclude_text_extensions: self.exclude_text_extensions.unwrap_or(defaults.exclude_text_extensions),
            include_patterns: self.include_patterns.unwrap_or(defaults.include_patterns),
            exclude_patterns: self.exclude_patterns.unwrap_or(defaults.exclude_patterns),
            hide_patterns: self.hide_patterns.unwrap_or(defaults.hide_patterns),
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

    #[test]
    fn test_partial_merge_local_overrides_global() {
        let global = PartialSettings {
            max_file_size: Some(1000),
            respect_gitignore: Some(true),
            ..Default::default()
        };
        let local = PartialSettings {
            max_file_size: Some(2000),
            ..Default::default()
        };
        let merged = global.merge(local);
        assert_eq!(merged.max_file_size, Some(2000));
        assert_eq!(merged.respect_gitignore, Some(true));
    }

    #[test]
    fn test_partial_merge_missing_fields_fallback() {
        let global = PartialSettings {
            show_hidden: Some(true),
            extra_text_extensions: Some(vec!["mdx".to_string()]),
            ..Default::default()
        };
        let local = PartialSettings {
            exclude_patterns: Some(vec!["target/**".to_string()]),
            ..Default::default()
        };
        let merged = global.merge(local);
        assert_eq!(merged.show_hidden, Some(true));
        assert_eq!(merged.extra_text_extensions, Some(vec!["mdx".to_string()]));
        assert_eq!(merged.exclude_patterns, Some(vec!["target/**".to_string()]));
    }

    #[test]
    fn test_partial_merge_empty_vec_overrides() {
        let global = PartialSettings {
            include_patterns: Some(vec!["*.rs".to_string()]),
            ..Default::default()
        };
        let local = PartialSettings {
            include_patterns: Some(vec![]),
            ..Default::default()
        };
        let merged = global.merge(local);
        assert_eq!(merged.include_patterns, Some(vec![]));
    }

    #[test]
    fn test_partial_resolve_defaults() {
        let partial = PartialSettings::default();
        let settings = partial.resolve();
        let defaults = Settings::default();
        assert_eq!(settings.max_file_size, defaults.max_file_size);
        assert_eq!(settings.max_clipboard_size, defaults.max_clipboard_size);
        assert_eq!(settings.respect_gitignore, defaults.respect_gitignore);
        assert_eq!(settings.show_hidden, defaults.show_hidden);
        assert!(settings.include_patterns.is_empty());
        assert!(settings.exclude_patterns.is_empty());
        assert!(settings.hide_patterns.is_empty());
    }

    #[test]
    fn test_load_with_local_merges_correctly() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create a "global" config by setting XDG_CONFIG_HOME
        let xdg_dir = root.join("xdg_config");
        std::fs::create_dir_all(&xdg_dir)?;
        let global_config = xdg_dir.join("gthr.toml");
        std::fs::write(&global_config, r#"
show_hidden = true
include_patterns = ["*.rs"]
"#)?;

        // Create a local config in project root
        let project_dir = root.join("project");
        std::fs::create_dir_all(&project_dir)?;
        std::fs::write(project_dir.join("gthr.toml"), r#"
exclude_patterns = ["target/**"]
"#)?;

        // Manually test the merge logic (can't override XDG in-process safely)
        let global = PartialSettings::load_from_file(&global_config);
        let local = PartialSettings::load_from_file(&project_dir.join("gthr.toml"));
        let settings = global.merge(local).resolve();

        assert!(settings.show_hidden);
        assert_eq!(settings.include_patterns, vec!["*.rs"]);
        assert_eq!(settings.exclude_patterns, vec!["target/**"]);
        // respect_gitignore should be default (true)
        assert!(settings.respect_gitignore);

        Ok(())
    }

    #[test]
    fn test_hide_patterns_in_settings_roundtrip() -> Result<()> {
        let mut settings = Settings::default();
        settings.hide_patterns = vec!["target/**".to_string(), "*.log".to_string()];

        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.toml");

        settings.save_to_file(&config_path)?;
        let loaded = Settings::load_from_file(&config_path)?;

        assert_eq!(loaded.hide_patterns, vec!["target/**", "*.log"]);

        Ok(())
    }

    #[test]
    fn test_partial_merge_hide_patterns() {
        let global = PartialSettings {
            hide_patterns: Some(vec!["*.log".to_string()]),
            ..Default::default()
        };
        let local = PartialSettings {
            hide_patterns: Some(vec!["target/**".to_string()]),
            ..Default::default()
        };
        let merged = global.merge(local);
        // Local replaces global
        assert_eq!(merged.hide_patterns, Some(vec!["target/**".to_string()]));
    }
}
