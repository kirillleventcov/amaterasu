use crate::{AmaterasuConfig, WipeMode};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub defaults: DefaultConfig,
    #[serde(default)]
    pub modes: ModeConfigs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_true")]
    pub verify: bool,
    #[serde(default = "default_true")]
    pub progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfigs {
    #[serde(default)]
    pub fast: ModeConfig,
    #[serde(default)]
    pub standard: ModeConfig,
    #[serde(default)]
    pub paranoid: ModeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    #[serde(default = "default_passes")]
    pub passes: usize,
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default = "default_sync")]
    pub sync: bool,
    #[serde(default)]
    pub verify_each_pass: bool,
    #[serde(default)]
    pub disable_cow: bool,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            verify: default_true(),
            progress: default_true(),
        }
    }
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self {
            passes: default_passes(),
            patterns: Vec::new(),
            sync: default_sync(),
            verify_each_pass: false,
            disable_cow: false,
        }
    }
}

impl Default for ModeConfigs {
    fn default() -> Self {
        Self {
            fast: ModeConfig {
                passes: 1,
                patterns: vec!["random".to_string()],
                sync: false,
                verify_each_pass: false,
                disable_cow: false,
            },
            standard: ModeConfig {
                passes: 3,
                patterns: vec![
                    "random".to_string(),
                    "zeros".to_string(),
                    "random".to_string(),
                ],
                sync: true,
                verify_each_pass: false,
                disable_cow: false,
            },
            paranoid: ModeConfig {
                passes: 7,
                patterns: vec![
                    "random".to_string(),
                    "0x55".to_string(),
                    "0xAA".to_string(),
                    "random".to_string(),
                    "ones".to_string(),
                    "zeros".to_string(),
                    "random".to_string(),
                ],
                sync: true,
                verify_each_pass: true,
                disable_cow: true,
            },
        }
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            defaults: DefaultConfig::default(),
            modes: ModeConfigs::default(),
        }
    }
}

fn default_mode() -> String {
    "standard".to_string()
}

fn default_true() -> bool {
    true
}

fn default_passes() -> usize {
    3
}

fn default_sync() -> bool {
    true
}

pub fn get_config_path() -> Result<PathBuf> {
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(xdg_config_home)
            .join("amaterasu")
            .join("config.toml"))
    } else if let Ok(home) = std::env::var("HOME") {
        Ok(PathBuf::from(home)
            .join(".config")
            .join("amaterasu")
            .join("config.toml"))
    } else {
        Err(anyhow::anyhow!("Could not determine config directory"))
    }
}

pub fn load_config() -> Result<ConfigFile> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        let config_str = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: ConfigFile = toml::from_str(&config_str)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

        Ok(config)
    } else {
        // Create default config file
        create_default_config(&config_path)?;
        Ok(ConfigFile::default())
    }
}

pub fn create_default_config(path: &Path) -> Result<()> {
    let default_config = ConfigFile::default();
    let config_str =
        toml::to_string_pretty(&default_config).context("Failed to serialize default config")?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    fs::write(path, config_str)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;

    println!("📝 Created default config file: {}", path.display());
    Ok(())
}

pub fn apply_config_to_amaterasu(
    _config: &ConfigFile,
    cli_config: AmaterasuConfig,
) -> AmaterasuConfig {
    // CLI arguments take precedence over config file
    AmaterasuConfig {
        verify: cli_config.verify,
        progress: cli_config.progress,
        force: cli_config.force,
        mode: cli_config.mode,
        wipe_metadata: cli_config.wipe_metadata,
        metadata_passes: cli_config.metadata_passes,
    }
}

pub fn parse_mode_from_config(mode_str: &str) -> WipeMode {
    match mode_str.to_lowercase().as_str() {
        "fast" => WipeMode::Fast,
        "standard" => WipeMode::Standard,
        "paranoid" => WipeMode::Paranoid,
        _ => WipeMode::Standard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config_creation() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.toml");

        create_default_config(&config_path)?;

        assert!(config_path.exists());

        let config = load_config_from_path(&config_path)?;
        assert_eq!(config.defaults.mode, "standard");
        assert!(config.defaults.verify);
        assert!(config.defaults.progress);

        Ok(())
    }

    fn load_config_from_path(path: &Path) -> Result<ConfigFile> {
        let config_str = fs::read_to_string(path)?;
        Ok(toml::from_str(&config_str)?)
    }

    #[test]
    fn test_mode_parsing() {
        assert!(matches!(parse_mode_from_config("fast"), WipeMode::Fast));
        assert!(matches!(
            parse_mode_from_config("STANDARD"),
            WipeMode::Standard
        ));
        assert!(matches!(
            parse_mode_from_config("paranoid"),
            WipeMode::Paranoid
        ));
        assert!(matches!(
            parse_mode_from_config("invalid"),
            WipeMode::Standard
        ));
    }
}
