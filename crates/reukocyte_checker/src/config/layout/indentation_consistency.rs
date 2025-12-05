const NORMAL: &str = "normal";
const INDENTED_INTERNAL_METHODS: &str = "indented internal methods";

/// Configuration for Layout/IndentationConsistency.
#[derive(Debug, Clone)]
pub struct IndentationConsistencyConfig {
    pub enforced_style: EnforcedStyle,
}
impl Default for IndentationConsistencyConfig {
    fn default() -> Self {
        Self {
            enforced_style: EnforcedStyle::default(),
        }
    }
}

/// Indentation style for Layout/IndentationConsistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnforcedStyle {
    #[default]
    Normal,
    IndentedInternalMethods,
}
impl EnforcedStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => NORMAL,
            Self::IndentedInternalMethods => INDENTED_INTERNAL_METHODS,
        }
    }
}
