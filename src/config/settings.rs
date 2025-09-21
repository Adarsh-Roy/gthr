use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub max_file_size: u64,
    pub respect_gitignore: bool,
    pub include_metadata: bool,
    pub include_line_numbers: bool,
    pub default_output_dir: Option<PathBuf>,
    pub file_extensions: FileExtensionSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExtensionSettings {
    pub text_extensions: Vec<String>,
    pub binary_extensions: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_file_size: 1024 * 1024, // 1MB
            respect_gitignore: true,
            include_metadata: true,
            include_line_numbers: false,
            default_output_dir: None,
            file_extensions: FileExtensionSettings::default(),
        }
    }
}

impl Default for FileExtensionSettings {
    fn default() -> Self {
        Self {
            text_extensions: vec![
                "txt".to_string(), "md".to_string(), "rs".to_string(), "py".to_string(),
                "js".to_string(), "ts".to_string(), "jsx".to_string(), "tsx".to_string(),
                "html".to_string(), "css".to_string(), "scss".to_string(), "sass".to_string(),
                "json".to_string(), "yaml".to_string(), "yml".to_string(), "toml".to_string(),
                "xml".to_string(), "csv".to_string(), "sql".to_string(), "sh".to_string(),
                "bash".to_string(), "zsh".to_string(), "fish".to_string(),
            ],
            binary_extensions: vec![
                "exe".to_string(), "dll".to_string(), "so".to_string(), "dylib".to_string(),
                "bin".to_string(), "obj".to_string(), "o".to_string(), "a".to_string(),
                "lib".to_string(), "png".to_string(), "jpg".to_string(), "jpeg".to_string(),
                "gif".to_string(), "bmp".to_string(), "ico".to_string(), "svg".to_string(),
                "pdf".to_string(), "zip".to_string(), "tar".to_string(), "gz".to_string(),
                "7z".to_string(), "rar".to_string(),
            ],
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

    pub fn get_config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("textingest").join("config.toml")
        } else {
            PathBuf::from(".textingestrc")
        }
    }

    pub fn load_or_default() -> Self {
        let config_path = Self::get_config_path();
        Self::load_from_file(&config_path).unwrap_or_default()
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