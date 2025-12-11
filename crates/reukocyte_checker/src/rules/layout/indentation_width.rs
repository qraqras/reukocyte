use crate::checker::Checker;
use crate::custom_nodes::AssignmentNode;
use crate::rule::*;
use reukocyte_macros::check;
use ruby_prism::*;

/// Get the config for this rule
#[inline]
fn config<'a>(checker: &'a Checker<'_>) -> &'a crate::config::layout::indentation_width::IndentationWidth {
    &checker.config().layout.indentation_width
}

/// Layout/IndentationWidth rule.
pub struct IndentationWidth;
impl Rule for IndentationWidth {
    const ID: RuleId = RuleId::Layout(LayoutRule::IndentationWidth);
}
#[check(StatementsNode)]
impl Check<StatementsNode<'_>> for IndentationWidth {
    fn check(_node: &StatementsNode, _checker: &mut Checker) {
        //
    }
}
#[check(AssignmentNode)]
impl Check<AssignmentNode<'_>> for IndentationWidth {
    fn check(_node: &AssignmentNode, _checker: &mut Checker) {
        //
    }
}
