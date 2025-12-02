//! Configuration parsing for .rubocop.yml

use serde::Deserialize;
use std::path::Path;

/// PreCop configuration
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub layout: LayoutConfig,
}

/// Layout-specific configuration
#[derive(Debug, Default, Deserialize)]
pub struct LayoutConfig {
    #[serde(default)]
    pub indentation_width: Option<IndentationWidthConfig>,

    #[serde(default)]
    pub trailing_whitespace: Option<CopConfig>,

    #[serde(default)]
    pub trailing_empty_lines: Option<TrailingEmptyLinesConfig>,
}

/// Base configuration for a cop
#[derive(Debug, Default, Deserialize)]
pub struct CopConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// Configuration for Layout/IndentationWidth
#[derive(Debug, Deserialize)]
pub struct IndentationWidthConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default = "default_indentation_width")]
    pub width: usize,
}

impl Default for IndentationWidthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            width: 2,
        }
    }
}

/// Configuration for Layout/TrailingEmptyLines
#[derive(Debug, Deserialize)]
pub struct TrailingEmptyLinesConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default = "default_final_newline")]
    pub final_newline: bool,
}

impl Default for TrailingEmptyLinesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            final_newline: true,
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_indentation_width() -> usize {
    2
}

fn default_final_newline() -> bool {
    true
}

impl Config {
    /// Load configuration from a .rubocop.yml file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from the current directory or use defaults
    pub fn load_or_default() -> Self {
        Self::load(".rubocop.yml").unwrap_or_default()
    }
}

/// Configuration loading errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    Parse(#[from] serde_yaml::Error),
}
