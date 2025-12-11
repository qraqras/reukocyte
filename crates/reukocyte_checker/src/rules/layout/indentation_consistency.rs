use crate::checker::Checker;
use crate::rule::Check;
use crate::rule::LayoutRule;
use crate::rule::Rule;
use crate::rule::RuleId;
use reukocyte_macros::check;
use ruby_prism::*;

/// Get the config for this rule
#[inline]
fn config<'a>(checker: &'a Checker<'_>) -> &'a crate::config::layout::indentation_consistency::IndentationConsistency {
    &checker.config().layout.indentation_consistency
}

/// Layout/IndentationConsistency rule.
pub struct IndentationConsistency;
impl Rule for IndentationConsistency {
    const ID: RuleId = RuleId::Layout(LayoutRule::IndentationConsistency);
}
#[check(StatementsNode)]
impl Check<StatementsNode<'_>> for IndentationConsistency {
    fn check(_node: &StatementsNode, _checker: &mut Checker) {
        //
    }
}
