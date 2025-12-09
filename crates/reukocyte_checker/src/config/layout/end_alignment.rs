/// Configuration for Layout/EndAlignment.
#[derive(Debug, Clone)]
pub struct EndAlignmentConfig {
    /// The style of alignment for `end` keywords.
    pub enforced_style_align_with: EnforcedStyleAlignWith,
}
impl Default for EndAlignmentConfig {
    fn default() -> Self {
        Self {
            enforced_style_align_with: EnforcedStyleAlignWith::default(),
        }
    }
}

/// Alignment style for Layout/EndAlignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnforcedStyleAlignWith {
    #[default]
    Keyword,
    Variable,
    StartOfLine,
}
