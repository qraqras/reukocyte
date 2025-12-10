use crate::config::serde_helpers::{deserialize_enabled, deserialize_severity};
use crate::diagnostic::Severity;
use serde::Deserialize;

/// Configuration for Layout/EndAlignment.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct EndAlignmentConfig {
    /// Whether this cop is enabled.
    #[serde(deserialize_with = "deserialize_enabled")]
    pub enabled: bool,
    /// Severity level for this cop.
    #[serde(deserialize_with = "deserialize_severity")]
    pub severity: Severity,
    /// The style of alignment for `end` keywords.
    pub enforced_style_align_with: EnforcedStyleAlignWith,
}
impl Default for EndAlignmentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Warning,
            enforced_style_align_with: EnforcedStyleAlignWith::default(),
        }
    }
}

/// Alignment style for Layout/EndAlignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcedStyleAlignWith {
    #[default]
    Keyword,
    Variable,
    StartOfLine,
}
