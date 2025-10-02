use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;
use crate::constants::DEFAULT_MAX_FILE_SIZE;

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
    #[serde(default = "default_include_metadata")]
    pub include_metadata: bool,
    #[serde(default = "default_include_line_numbers")]
    pub include_line_numbers: bool,
    #[serde(default)]
    pub default_output_dir: Option<PathBuf>,
    #[serde(default)]
    pub file_extensions: FileExtensionSettings,
}

fn default_max_file_size() -> u64 { DEFAULT_MAX_FILE_SIZE }
fn default_max_clipboard_size() -> usize { 2 * 1024 * 1024 }
fn default_respect_gitignore() -> bool { true }
fn default_show_hidden() -> bool { false }
fn default_include_metadata() -> bool { true }
fn default_include_line_numbers() -> bool { false }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExtensionSettings {
    #[serde(default = "default_text_extensions")]
    pub text_extensions: Vec<String>,
    #[serde(default = "default_binary_extensions")]
    pub binary_extensions: Vec<String>,
}

fn default_text_extensions() -> Vec<String> {
    vec![
        "txt".to_string(), "md".to_string(), "rs".to_string(), "py".to_string(),
        "js".to_string(), "ts".to_string(), "jsx".to_string(), "tsx".to_string(),
        "html".to_string(), "css".to_string(), "scss".to_string(), "sass".to_string(),
        "json".to_string(), "yaml".to_string(), "yml".to_string(), "toml".to_string(),
        "xml".to_string(), "csv".to_string(), "sql".to_string(), "sh".to_string(),
        "bash".to_string(), "zsh".to_string(), "fish".to_string(),
    ]
}

fn default_binary_extensions() -> Vec<String> {
    vec![
        "exe".to_string(), "dll".to_string(), "so".to_string(), "dylib".to_string(),
        "bin".to_string(), "obj".to_string(), "o".to_string(), "a".to_string(),
        "lib".to_string(), "png".to_string(), "jpg".to_string(), "jpeg".to_string(),
        "gif".to_string(), "bmp".to_string(), "ico".to_string(), "svg".to_string(),
        "pdf".to_string(), "zip".to_string(), "tar".to_string(), "gz".to_string(),
        "7z".to_string(), "rar".to_string(),
    ]
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_file_size: default_max_file_size(),
            max_clipboard_size: default_max_clipboard_size(),
            respect_gitignore: default_respect_gitignore(),
            show_hidden: default_show_hidden(),
            include_metadata: default_include_metadata(),
            include_line_numbers: default_include_line_numbers(),
            default_output_dir: None,
            file_extensions: FileExtensionSettings::default(),
        }
    }
}

impl Default for FileExtensionSettings {
    fn default() -> Self {
        Self {
            text_extensions: default_text_extensions(),
            binary_extensions: default_binary_extensions(),
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

    pub fn get_global_config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join(".gthr.toml")
        } else if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(".config").join(".gthr.toml")
        } else {
            PathBuf::from(".gthr.toml")
        }
    }

    pub fn get_project_config_path(project_root: &std::path::Path) -> PathBuf {
        project_root.join(".gthr.toml")
    }

    pub fn load_or_default() -> Self {
        Self::load_with_project_root(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    pub fn load_with_project_root(project_root: &std::path::Path) -> Self {
        // Start with default settings
        let mut settings = Self::default();

        // Load global config first (lower priority)
        let global_config_path = Self::get_global_config_path();
        if let Ok(global_settings) = Self::load_from_file(&global_config_path) {
            settings = global_settings;
        }

        // Load project-specific config second (higher priority - overrides global)
        let project_config_path = Self::get_project_config_path(project_root);
        if let Ok(project_settings) = Self::load_from_file(&project_config_path) {
            // Project settings override global settings (serde handles defaults for missing fields)
            settings = Self::merge_settings(settings, project_settings);
        }

        settings
    }

    fn merge_settings(mut global: Settings, project: Settings) -> Settings {
        // Only override non-default values from project config
        if project.max_file_size != default_max_file_size() {
            global.max_file_size = project.max_file_size;
        }
        if project.max_clipboard_size != default_max_clipboard_size() {
            global.max_clipboard_size = project.max_clipboard_size;
        }
        if project.respect_gitignore != default_respect_gitignore() {
            global.respect_gitignore = project.respect_gitignore;
        }
        if project.show_hidden != default_show_hidden() {
            global.show_hidden = project.show_hidden;
        }
        if project.include_metadata != default_include_metadata() {
            global.include_metadata = project.include_metadata;
        }
        if project.include_line_numbers != default_include_line_numbers() {
            global.include_line_numbers = project.include_line_numbers;
        }
        if project.default_output_dir.is_some() {
            global.default_output_dir = project.default_output_dir;
        }
        // For file extensions, if they're explicitly set in project config, use them
        if project.file_extensions.text_extensions != default_text_extensions() {
            global.file_extensions.text_extensions = project.file_extensions.text_extensions;
        }
        if project.file_extensions.binary_extensions != default_binary_extensions() {
            global.file_extensions.binary_extensions = project.file_extensions.binary_extensions;
        }
        global
    }

    /// Format clipboard size for user-facing messages
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
}