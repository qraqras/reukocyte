use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/IndentationStyle.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct IndentationStyle {
    /// Base configuration (enabled, severity, exclude, include).
    #[serde(flatten)]
    pub base: BaseCopConfig,
    /// Enforced style for indentation.
    pub enforced_style: EnforcedStyle,
    /// Width of indentation (for tab-to-spaces conversion).
    pub indentation_width: usize,
}

impl Default for IndentationStyle {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::default(),
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
