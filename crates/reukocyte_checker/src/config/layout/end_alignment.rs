use crate::config::BaseCopConfig;
use crate::diagnostic::Severity;
use serde::Deserialize;

/// Configuration for Layout/EndAlignment.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct EndAlignment {
    #[serde(flatten)]
    pub base: BaseCopConfig,
    pub enforced_style_align_with: EnforcedStyleAlignWith,
}
impl Default for EndAlignment {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::with_severity(Severity::Warning),
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
    StartOfLine,
    Variable,
}
