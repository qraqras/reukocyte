use crate::diagnostic::Severity;

/// Configuration for Layout/DefEndAlignment.
#[derive(Debug, Clone)]
pub struct DefEndAlignmentConfig {
    pub enforced_style_align_with: EnforcedStyleAlignWith,
    pub severity: Severity,
}
impl Default for DefEndAlignmentConfig {
    fn default() -> Self {
        Self {
            enforced_style_align_with: EnforcedStyleAlignWith::default(),
            severity: Severity::Warning,
        }
    }
}

/// Alignment style for Layout/DefEndAlignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnforcedStyleAlignWith {
    #[default]
    StartOfLine,
    Def,
}
