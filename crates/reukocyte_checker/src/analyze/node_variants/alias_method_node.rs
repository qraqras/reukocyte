#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::AliasMethodNode;

/// Run lint rules over a [`AliasMethodNode`] syntax node.
pub(crate) fn alias_method_node(node: &AliasMethodNode, checker: &mut Checker) {
    // TODO: Add rules for AliasMethodNode
}
