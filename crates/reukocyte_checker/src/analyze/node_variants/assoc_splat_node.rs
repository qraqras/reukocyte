#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::AssocSplatNode;

/// Run lint rules over a [`AssocSplatNode`] syntax node.
pub(crate) fn assoc_splat_node(node: &AssocSplatNode, checker: &mut Checker) {
    // TODO: Add rules for AssocSplatNode
}
