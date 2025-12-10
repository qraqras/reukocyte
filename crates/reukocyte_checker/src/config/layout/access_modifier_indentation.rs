use crate::config::serde_helpers::{deserialize_enabled, deserialize_severity};
use crate::diagnostic::Severity;
use serde::Deserialize;

/// Configuration for Layout/AccessModifierIndentation.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct AccessModifierIndentation {
    /// Whether this cop is enabled.
    #[serde(deserialize_with = "deserialize_enabled")]
    pub enabled: bool,
    /// Severity level for this cop.
    #[serde(deserialize_with = "deserialize_severity")]
    pub severity: Severity,
    pub enforced_style: EnforcedStyle,
    pub indentation_width: Option<usize>,
}
impl Default for AccessModifierIndentation {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Convention,
            enforced_style: EnforcedStyle::default(),
            indentation_width: None,
        }
    }
}

/// Alignment style for Layout/AccessModifierIndentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcedStyle {
    #[default]
    Indent,
    Outdent,
}
