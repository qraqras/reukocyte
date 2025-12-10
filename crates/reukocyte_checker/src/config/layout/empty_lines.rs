use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/EmptyLines.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct EmptyLines {
    /// Base configuration (enabled, severity, exclude, include).
    #[serde(flatten)]
    pub base: BaseCopConfig,
}

impl Default for EmptyLines {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::default(),
        }
    }
}
