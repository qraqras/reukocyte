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

/// Layout rule configurations.
#[derive(Debug, Clone, Default)]
pub struct LayoutConfig {
    pub access_modifier_indentation: access_modifier_indentation::AccessModifierIndentation,
    pub begin_end_alignment: begin_end_alignment::BeginEndAlignment,
    pub def_end_alignment: def_end_alignment::DefEndAlignment,
    pub empty_lines: empty_lines::EmptyLines,
    pub end_alignment: end_alignment::EndAlignment,
    pub indentation_consistency: indentation_consistency::IndentationConsistency,
    pub indentation_style: indentation_style::IndentationStyle,
    pub indentation_width: indentation_width::IndentationWidth,
    pub leading_empty_lines: leading_empty_lines::LeadingEmptyLines,
    pub trailing_empty_lines: trailing_empty_lines::TrailingEmptyLines,
    pub trailing_whitespace: trailing_whitespace::TrailingWhitespace,
}
