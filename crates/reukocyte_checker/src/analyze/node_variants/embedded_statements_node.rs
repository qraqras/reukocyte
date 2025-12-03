#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::EmbeddedStatementsNode;

/// Run lint rules over a [`EmbeddedStatementsNode`] syntax node.
pub(crate) fn embedded_statements_node(node: &EmbeddedStatementsNode, checker: &mut Checker) {
    // TODO: Add rules for EmbeddedStatementsNode
}
