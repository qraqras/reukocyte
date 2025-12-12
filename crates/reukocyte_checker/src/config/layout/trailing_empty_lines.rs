use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/TrailingEmptyLines.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct TrailingEmptyLines {
    #[serde(flatten)]
    pub base: BaseCopConfig,
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
    FinalBlankLine,
    #[default]
    FinalNewline,
}
