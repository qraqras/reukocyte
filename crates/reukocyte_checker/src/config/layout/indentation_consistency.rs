use crate::config::serde_helpers::{deserialize_enabled, deserialize_severity};
use crate::diagnostic::Severity;
use serde::Deserialize;

const NORMAL: &str = "normal";
const INDENTED_INTERNAL_METHODS: &str = "indented_internal_methods";

/// Configuration for Layout/IndentationConsistency.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct IndentationConsistencyConfig {
    /// Whether this cop is enabled.
    #[serde(deserialize_with = "deserialize_enabled")]
    pub enabled: bool,
    /// Severity level for this cop.
    #[serde(deserialize_with = "deserialize_severity")]
    pub severity: Severity,
    pub enforced_style: EnforcedStyle,
}
impl Default for IndentationConsistencyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Convention,
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
