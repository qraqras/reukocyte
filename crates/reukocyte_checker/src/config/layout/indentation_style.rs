use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/IndentationStyle.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct IndentationStyle {
    #[serde(flatten)]
    pub base: BaseCopConfig,
    pub enforced_style: EnforcedStyle,
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
    #[default]
    Spaces,
    Tabs,
}
