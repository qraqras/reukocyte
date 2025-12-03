#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::BackReferenceReadNode;

/// Run lint rules over a [`BackReferenceReadNode`] syntax node.
pub(crate) fn back_reference_read_node(node: &BackReferenceReadNode, checker: &mut Checker) {
    // TODO: Add rules for BackReferenceReadNode
}
