#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::GlobalVariableTargetNode;

/// Run lint rules over a [`GlobalVariableTargetNode`] syntax node.
pub(crate) fn global_variable_target_node(node: &GlobalVariableTargetNode, checker: &mut Checker) {
    // TODO: Add rules for GlobalVariableTargetNode
}
