#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::StatementsNode;

/// Run lint rules over a [`StatementsNode`] syntax node.
pub(crate) fn statements_node(node: &StatementsNode, checker: &mut Checker) {
    // TODO: Add rules for StatementsNode
}
