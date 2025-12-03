#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantReadNode;

/// Run lint rules over a [`ConstantReadNode`] syntax node.
pub(crate) fn constant_read_node(node: &ConstantReadNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantReadNode
}
