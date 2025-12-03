#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::IntegerNode;

/// Run lint rules over a [`IntegerNode`] syntax node.
pub(crate) fn integer_node(node: &IntegerNode, checker: &mut Checker) {
    // TODO: Add rules for IntegerNode
}
