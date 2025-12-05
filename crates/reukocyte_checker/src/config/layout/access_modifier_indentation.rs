/// Configuration for Layout/AccessModifierIndentation.
#[derive(Debug, Clone)]
pub struct AccessModifierIndentationConfig {
    pub enforced_style: EnforcedStyle,
    pub indentation_width: Option<usize>,
}

impl Default for AccessModifierIndentationConfig {
    fn default() -> Self {
        Self {
            enforced_style: EnforcedStyle::default(),
            indentation_width: None,
        }
    }
}

/// Alignment style for Layout/AccessModifierIndentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnforcedStyle {
    #[default]
    Indent,
    Outdent,
}
