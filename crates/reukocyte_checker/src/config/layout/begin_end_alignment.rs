use crate::diagnostic::Severity;

/// Configuration for Layout/BeginEndAlignment.
#[derive(Debug, Clone)]
pub struct BeginEndAlignmentConfig {
    pub enforced_style_align_with: EnforcedStyleAlignWith,
    pub severity: Severity,
}
impl Default for BeginEndAlignmentConfig {
    fn default() -> Self {
        Self {
            enforced_style_align_with: EnforcedStyleAlignWith::default(),
            severity: Severity::default(),
        }
    }
}

/// Alignment style for Layout/AccessModifierIndentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnforcedStyleAlignWith {
    #[default]
    StartOfLine,
    Begin,
}
