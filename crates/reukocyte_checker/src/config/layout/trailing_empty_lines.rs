use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/TrailingEmptyLines.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct TrailingEmptyLines {
    /// Base configuration (enabled, severity, exclude, include).
    #[serde(flatten)]
    pub base: BaseCopConfig,
    /// Enforced style for trailing empty lines.
    pub enforced_style: EnforcedStyle,
}

impl Default for TrailingEmptyLines {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::default(),
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
