use crate::config::BaseCopConfig;
use serde::Deserialize;

const NORMAL: &str = "normal";
const INDENTED_INTERNAL_METHODS: &str = "indented_internal_methods";

/// Configuration for Layout/IndentationConsistency.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct IndentationConsistency {
    /// Base configuration (enabled, severity, exclude, include).
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
    #[default]
    Normal,
    IndentedInternalMethods,
}
impl EnforcedStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => NORMAL,
            Self::IndentedInternalMethods => INDENTED_INTERNAL_METHODS,
        }
    }
}
