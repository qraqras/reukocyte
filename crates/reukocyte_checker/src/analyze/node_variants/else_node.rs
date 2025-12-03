#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ElseNode;

/// Run lint rules over a [`ElseNode`] syntax node.
pub(crate) fn else_node(node: &ElseNode, checker: &mut Checker) {
    // TODO: Add rules for ElseNode
}
