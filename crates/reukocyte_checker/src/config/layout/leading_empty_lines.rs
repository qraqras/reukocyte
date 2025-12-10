use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/LeadingEmptyLines.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct LeadingEmptyLines {
    /// Base configuration (enabled, severity, exclude, include).
    #[serde(flatten)]
    pub base: BaseCopConfig,
}

impl Default for LeadingEmptyLines {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::default(),
        }
    }
}
