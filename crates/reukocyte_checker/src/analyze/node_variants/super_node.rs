#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::SuperNode;

/// Run lint rules over a [`SuperNode`] syntax node.
pub(crate) fn super_node(node: &SuperNode, checker: &mut Checker) {
    // TODO: Add rules for SuperNode
}
