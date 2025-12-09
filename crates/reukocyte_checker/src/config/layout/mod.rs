pub mod access_modifier_indentation;
pub mod begin_end_alignment;
pub mod def_end_alignment;
pub mod end_alignment;
pub mod indentation_consistency;
pub mod indentation_width;

/// Layout cop configurations.
#[derive(Debug, Clone, Default)]
pub struct LayoutConfig {
    pub access_modifier_indentation: access_modifier_indentation::AccessModifierIndentationConfig,
    pub begin_end_alignment: begin_end_alignment::BeginEndAlignmentConfig,
    pub def_end_alignment: def_end_alignment::DefEndAlignmentConfig,
    pub end_alignment: end_alignment::EndAlignmentConfig,
    pub indentation_consistency: indentation_consistency::IndentationConsistencyConfig,
    pub indentation_width: indentation_width::IndentationWidthConfig,
}
