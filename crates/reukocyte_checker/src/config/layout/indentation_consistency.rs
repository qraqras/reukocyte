use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/IndentationConsistency.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct IndentationConsistency {
    #[serde(flatten)]
    pub base: BaseCopConfig,
    pub enforced_style: EnforcedStyle,
}

impl Default for IndentationConsistency {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::default(),
            enforced_style: EnforcedStyle::default(),
        }
    }
}

/// Indentation style for Layout/IndentationConsistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcedStyle {
    IndentedInternalMethods,
    #[default]
    Normal,
}
