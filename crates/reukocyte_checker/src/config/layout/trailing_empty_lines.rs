use crate::config::serde_helpers::{deserialize_enabled, deserialize_severity};
use crate::diagnostic::Severity;
use serde::Deserialize;

/// Configuration for Layout/TrailingEmptyLines.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct TrailingEmptyLinesConfig {
    /// Whether this cop is enabled.
    #[serde(deserialize_with = "deserialize_enabled")]
    pub enabled: bool,
    /// Severity level for this cop.
    #[serde(deserialize_with = "deserialize_severity")]
    pub severity: Severity,
    /// Enforced style for trailing empty lines.
    pub enforced_style: EnforcedStyle,
}
impl Default for TrailingEmptyLinesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Convention,
            enforced_style: EnforcedStyle::default(),
        }
    }
}

/// Enforced style for trailing empty lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcedStyle {
    /// Require exactly one final newline (no trailing blank lines).
    #[default]
    FinalNewline,
    /// Require one blank line followed by a final newline.
    FinalBlankLine,
}
