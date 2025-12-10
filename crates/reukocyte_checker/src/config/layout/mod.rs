pub mod access_modifier_indentation;
pub mod begin_end_alignment;
pub mod def_end_alignment;
pub mod empty_lines;
pub mod end_alignment;
pub mod indentation_consistency;
pub mod indentation_style;
pub mod indentation_width;
pub mod leading_empty_lines;
pub mod trailing_empty_lines;
pub mod trailing_whitespace;

// Re-export all config types for convenience
pub use access_modifier_indentation::AccessModifierIndentationConfig;
pub use begin_end_alignment::BeginEndAlignmentConfig;
pub use def_end_alignment::DefEndAlignmentConfig;
pub use empty_lines::EmptyLinesConfig;
pub use end_alignment::EndAlignmentConfig;
pub use indentation_consistency::IndentationConsistencyConfig;
pub use indentation_style::IndentationStyleConfig;
pub use indentation_width::IndentationWidthConfig;
pub use leading_empty_lines::LeadingEmptyLinesConfig;
pub use trailing_empty_lines::TrailingEmptyLinesConfig;
pub use trailing_whitespace::TrailingWhitespaceConfig;

/// Layout cop configurations.
#[derive(Debug, Clone, Default)]
pub struct LayoutConfig {
    pub access_modifier_indentation: access_modifier_indentation::AccessModifierIndentationConfig,
    pub begin_end_alignment: begin_end_alignment::BeginEndAlignmentConfig,
    pub def_end_alignment: def_end_alignment::DefEndAlignmentConfig,
    pub empty_lines: empty_lines::EmptyLinesConfig,
    pub end_alignment: end_alignment::EndAlignmentConfig,
    pub indentation_consistency: indentation_consistency::IndentationConsistencyConfig,
    pub indentation_style: indentation_style::IndentationStyleConfig,
    pub indentation_width: indentation_width::IndentationWidthConfig,
    pub leading_empty_lines: leading_empty_lines::LeadingEmptyLinesConfig,
    pub trailing_empty_lines: trailing_empty_lines::TrailingEmptyLinesConfig,
    pub trailing_whitespace: trailing_whitespace::TrailingWhitespaceConfig,
}
