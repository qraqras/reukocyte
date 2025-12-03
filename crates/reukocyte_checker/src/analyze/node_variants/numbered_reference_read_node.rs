#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::NumberedReferenceReadNode;

/// Run lint rules over a [`NumberedReferenceReadNode`] syntax node.
pub(crate) fn numbered_reference_read_node(node: &NumberedReferenceReadNode, checker: &mut Checker) {
    // TODO: Add rules for NumberedReferenceReadNode
}
