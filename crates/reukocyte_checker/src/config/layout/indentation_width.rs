use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/IndentationWidth.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct IndentationWidth {
    /// Base configuration (enabled, severity, exclude, include).
    #[serde(flatten)]
    pub base: BaseCopConfig,
    pub width: i32,
    pub allowed_patterns: Vec<i32>,
}

impl Default for IndentationWidth {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::default(),
            width: 2,
            allowed_patterns: Vec::new(),
        }
    }
}
