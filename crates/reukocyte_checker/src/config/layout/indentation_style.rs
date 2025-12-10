use crate::config::serde_helpers::{deserialize_enabled, deserialize_severity};
use crate::diagnostic::Severity;
use serde::Deserialize;

/// Configuration for Layout/IndentationStyle.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct IndentationStyle {
    /// Whether this cop is enabled.
    #[serde(deserialize_with = "deserialize_enabled")]
    pub enabled: bool,
    /// Severity level for this cop.
    #[serde(deserialize_with = "deserialize_severity")]
    pub severity: Severity,
    /// Enforced style for indentation.
    pub enforced_style: EnforcedStyle,
    /// Width of indentation (for tab-to-spaces conversion).
    pub indentation_width: usize,
}
impl Default for IndentationStyle {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Convention,
            enforced_style: EnforcedStyle::default(),
            indentation_width: 2,
        }
    }
}

/// Enforced style for indentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcedStyle {
    /// Use spaces for indentation.
    #[default]
    Spaces,
    /// Use tabs for indentation.
    Tabs,
}
