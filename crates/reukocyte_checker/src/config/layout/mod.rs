pub mod access_modifier_indentation;
pub mod def_end_alignment;
pub mod end_alignment;
pub mod indentation_consistency;
pub mod indentation_width;

/// Layout cop configurations.
#[derive(Debug, Clone, Default)]
pub struct LayoutConfig {
    pub end_alignment: end_alignment::EndAlignmentConfig,
    pub indentation_width: indentation_width::IndentationWidthConfig,
    pub indentation_consistency: indentation_consistency::IndentationConsistencyConfig,
    pub def_end_alignment: def_end_alignment::DefEndAlignmentConfig,
    pub access_modifier_indentation: access_modifier_indentation::AccessModifierIndentationConfig,
}
