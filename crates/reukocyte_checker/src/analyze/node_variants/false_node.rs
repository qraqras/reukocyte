#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::FalseNode;

/// Run lint rules over a [`FalseNode`] syntax node.
pub(crate) fn false_node(node: &FalseNode, checker: &mut Checker) {
    // TODO: Add rules for FalseNode
}
