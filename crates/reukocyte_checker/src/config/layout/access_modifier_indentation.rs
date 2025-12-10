use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/AccessModifierIndentation.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct AccessModifierIndentation {
    /// Base configuration (enabled, severity, exclude, include).
    #[serde(flatten)]
    pub base: BaseCopConfig,
    pub enforced_style: EnforcedStyle,
    pub indentation_width: Option<usize>,
}

impl Default for AccessModifierIndentation {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::default(),
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
