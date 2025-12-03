#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::PinnedVariableNode;

/// Run lint rules over a [`PinnedVariableNode`] syntax node.
pub(crate) fn pinned_variable_node(node: &PinnedVariableNode, checker: &mut Checker) {
    // TODO: Add rules for PinnedVariableNode
}
