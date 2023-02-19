use anyhow::*;
use optional_struct::*;
use serde::Deserialize;
use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

const CONFIG_FILE_NAME: &str = "config.yaml";
const DEFAULT_CONFIG_FILE_CONTENTS: &str = include_str!("../../assets/default_config.yaml");

const DEFAULT_CAPTURE_INTERVAL: f64 = 3.0;

#[optional_struct]
#[derive(Debug, Deserialize)]
pub struct DesktopDyeConfig {
    pub screen_id: Option<u32>,
    pub ha_endpoint: String,
    pub ha_token: String,
    pub ha_target_entity_id: String,
    pub capture_interval: f64,
    pub mode: ColorSelectionMode,
    pub hue_shift: f64,
}

impl DesktopDyeConfig {
    /// Returns the path to the config file, which is located in the user's home directory.
    ///
    /// `~/.desktop_dye/config.yaml`
    pub fn get_file_path() -> Result<PathBuf> {
        let home_dir =
            dirs::home_dir().context("Failed to get home directory while fetching config")?;
        let config_dir = home_dir.join(".desktop_dye");
        let config_file = config_dir.join(CONFIG_FILE_NAME);

        Ok(config_file)
    }

    /// Indicates whether the config file exists already.
    ///
    /// If not, it can be created using `DesktopDyeConfig::create_default()`.
    pub fn exists() -> Result<bool> {
        let path = Self::get_file_path()?;

        Ok(path.exists())
    }

    /// Creates a default config file in the user's home directory.
    ///
    /// The contents are copied from the default config file in `assets/default_config.yaml`.
    ///
    /// If the config file already exists, an error is returned.
    pub fn create_default() -> Result<()> {
        let path = Self::get_file_path()?;

        if path.exists() {
            return Err(anyhow!("Config file already exists"));
        }

        let config_dir = path.parent().unwrap();
        std::fs::create_dir_all(config_dir).context("Failed to create config directory")?;

        std::fs::write(&path, DEFAULT_CONFIG_FILE_CONTENTS)
            .context("Failed to write default config file")?;

        Ok(())
    }

    /// Retrieves the config from the config file.
    ///
    /// If the config file does not exist, an error is returned.
    pub fn get() -> Result<Self> {
        let path = Self::get_file_path()?;

        if !path.exists() {
            return Err(anyhow!("Config file does not exist"));
        }

        let file = std::fs::File::open(&path).context("Failed to open config file")?;
        let optional_config = serde_yaml::from_reader::<_, OptionalDesktopDyeConfig>(file)
            .context("Failed to read config file")?;

        let validation_errors = Self::validate_optional_config(&optional_config);
        if !validation_errors.is_empty() {
            let errors = validation_errors
                .into_iter()
                .map(|line| format!("  - {}", line))
                .collect::<Vec<_>>()
                .join("\n");

            return Err(anyhow!(
                "Config file is invalid. Please fix the following errors:\n{}",
                errors
            ));
        }

        Ok(DesktopDyeConfig {
            screen_id: optional_config.screen_id,
            ha_endpoint: optional_config.ha_endpoint.unwrap(),
            ha_token: optional_config.ha_token.unwrap(),
            ha_target_entity_id: optional_config.ha_target_entity_id.unwrap(),
            capture_interval: optional_config
                .capture_interval
                .unwrap_or(DEFAULT_CAPTURE_INTERVAL),
            mode: optional_config.mode.unwrap_or(ColorSelectionMode::Default),
            hue_shift: optional_config.hue_shift.unwrap_or(45.0),
        })
    }

    /// Validates the given optional config and returns a list of errors.
    ///
    /// If the config is valid, an empty list is returned.
    fn validate_optional_config(optional_config: &OptionalDesktopDyeConfig) -> Vec<String> {
        let mut errors = Vec::new();

        if optional_config.ha_endpoint.is_none() {
            errors.push("Missing Home Assistant endpoint in config file".to_string());
        } else {
            let ha_endpoint = optional_config.ha_endpoint.as_ref().unwrap();
            if !ha_endpoint.starts_with("http://") && !ha_endpoint.starts_with("https://") {
                errors.push(format!(
                    "Home Assistant endpoint must start with http:// or https://. Found \"{}\"",
                    ha_endpoint
                ));
            }

            if ha_endpoint.ends_with('/') {
                errors.push(format!(
                    "Home Assistant endpoint must not end with /. Found \"{}\"",
                    ha_endpoint
                ));
            }

            if ha_endpoint.chars().filter(|c| *c == ':').count() != 2 {
                errors.push(format!(
                    "Home Assistant endpoint must contain exactly two \':\', before the port. Found \"{}\"",
                    ha_endpoint
                ));
            }
        }

        if optional_config.ha_token.is_none() {
            errors.push("Missing Home Assistant token in config file".to_string());
        }

        if optional_config.ha_target_entity_id.is_none() {
            errors.push("Missing Home Assistant target entity ID in config file".to_string());
        }

        if let Some(capture_interval) = optional_config.capture_interval {
            if capture_interval <= 0.0 {
                errors.push(format!(
                    "Capture interval must be greater than 0. Found {}",
                    capture_interval
                ));
            } else if let Err(duration_err) =
                std::time::Duration::try_from_secs_f64(capture_interval)
            {
                errors.push(format!(
                    "Capture interval invalid. Cannot time out for amount of seconds provided. Found {}. Error: {}",
                    capture_interval,
                    duration_err
                ));
            }
        }

        errors
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum ColorSelectionMode {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "brightness")]
    Brightness,
    #[serde(rename = "hue_shift")]
    HueShift,
}

impl Display for ColorSelectionMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorSelectionMode::Default => write!(f, "Default"),
            ColorSelectionMode::Brightness => write!(f, "Brightness"),
            ColorSelectionMode::HueShift => write!(f, "Hue Shift"),
        }
    }
}
