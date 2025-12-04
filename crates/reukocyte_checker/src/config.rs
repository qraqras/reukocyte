use crate::diagnostic::Severity;

/// The main configuration struct.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub layout: LayoutConfig,
}

/// Layout cop configurations.
#[derive(Debug, Clone, Default)]
pub struct LayoutConfig {
    pub end_alignment: LayoutEndAlignmentConfig,
    pub indentation_width: LayoutIndentationWidthConfig,
    pub indentation_consistency: LayoutIndentationConsistencyConfig,
    pub def_end_alignment: LayoutDefEndAlignmentConfig,
}

/// Configuration for Layout/EndAlignment.
///
/// Controls how `end` keywords should be aligned.
#[derive(Debug, Clone)]
pub struct LayoutEndAlignmentConfig {
    /// The style of alignment for `end` keywords.
    pub enforced_style_align_with: AlignWith,
}
impl Default for LayoutEndAlignmentConfig {
    fn default() -> Self {
        Self {
            enforced_style_align_with: AlignWith::default(),
        }
    }
}

/// Configuration for Layout/IndentationWidth.
#[derive(Debug, Clone)]
pub struct LayoutIndentationWidthConfig {
    pub width: i32,
    pub allowed_patterns: Vec<i32>,
}
impl Default for LayoutIndentationWidthConfig {
    fn default() -> Self {
        Self {
            width: 2,
            allowed_patterns: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutIndentationConsistencyConfig {
    pub enforced_style: EnforcedStyle,
}
impl Default for LayoutIndentationConsistencyConfig {
    fn default() -> Self {
        Self {
            enforced_style: EnforcedStyle::default(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct LayoutDefEndAlignmentConfig {
    pub enforced_style_align_with: EnforcedStyleAlignWith,
    pub severity: Severity,
}
impl Default for LayoutDefEndAlignmentConfig {
    fn default() -> Self {
        Self {
            enforced_style_align_with: EnforcedStyleAlignWith::default(),
            severity: Severity::Warning,
        }
    }
}

/// Alignment style for `end` keywords (Layout/EndAlignment).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignWith {
    #[default]
    Keyword,
    Variable,
    StartOfLine,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnforcedStyle {
    #[default]
    Normal,
    IndentedInternalMethods,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnforcedStyleAlignWith {
    #[default]
    StartOfLine,
    Def,
}
