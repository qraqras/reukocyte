use crate::checker::Checker;
use crate::config::layout::end_alignment::EnforcedStyleAlignWith;
use ruby_prism::Location;

pub fn should_variable_alignment(
    whole_expression: Location,
    rhs: Location,
    end_alignment_style: EnforcedStyleAlignWith,
    checker: &Checker,
) -> bool {
    match end_alignment_style {
        EnforcedStyleAlignWith::Keyword => false,
        _ => !has_break_before_keyword(whole_expression, rhs, checker),
    }
}

pub fn has_break_before_keyword(whole_expression: Location, rhs: Location, checker: &Checker) -> bool {
    let line_index = checker.line_index();
    let rhs_start_line = line_index.line_number(rhs.start_offset());
    let expression_start_line = line_index.line_number(whole_expression.start_offset());

    rhs_start_line > expression_start_line
}
