#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::FloatNode;

/// Run lint rules over a [`FloatNode`] syntax node.
pub(crate) fn float_node(node: &FloatNode, checker: &mut Checker) {
    // TODO: Add rules for FloatNode
}
