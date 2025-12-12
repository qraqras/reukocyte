use crate::config::BaseCopConfig;
use crate::diagnostic::Severity;
use serde::Deserialize;

/// Configuration for Layout/DefEndAlignment.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct DefEndAlignment {
    #[serde(flatten)]
    pub base: BaseCopConfig,
    pub enforced_style_align_with: EnforcedStyleAlignWith,
}
impl Default for DefEndAlignment {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::with_severity(Severity::Warning),
            enforced_style_align_with: EnforcedStyleAlignWith::default(),
        }
    }
}

/// Alignment style for Layout/DefEndAlignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcedStyleAlignWith {
    Def,
    #[default]
    StartOfLine,
}
