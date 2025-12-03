#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::AliasGlobalVariableNode;

/// Run lint rules over a [`AliasGlobalVariableNode`] syntax node.
pub(crate) fn alias_global_variable_node(node: &AliasGlobalVariableNode, checker: &mut Checker) {
    // TODO: Add rules for AliasGlobalVariableNode
}
