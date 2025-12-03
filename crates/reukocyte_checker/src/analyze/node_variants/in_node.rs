#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InNode;

/// Run lint rules over a [`InNode`] syntax node.
pub(crate) fn in_node(node: &InNode, checker: &mut Checker) {
    // TODO: Add rules for InNode
}
