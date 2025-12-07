#![allow(unused_variables)]
use crate::checker::Checker;
#[allow(unused_imports)]
use crate::rules::layout::end_alignment::EndAlignment;
use crate::rules::layout::indentation_width::IndentationWidth;
use crate::run_rules;
use ruby_prism::StatementsNode;

/// Run lint rules over a [`StatementsNode`] syntax node.
pub(crate) fn statements_node(node: &StatementsNode, checker: &mut Checker) {
    // Run rules using the trait-based dispatch
    run_rules!(node, checker, StatementsNode<'_>, [EndAlignment, IndentationWidth,]);
}
