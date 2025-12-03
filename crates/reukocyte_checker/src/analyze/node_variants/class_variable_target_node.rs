#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ClassVariableTargetNode;

/// Run lint rules over a [`ClassVariableTargetNode`] syntax node.
pub(crate) fn class_variable_target_node(node: &ClassVariableTargetNode, checker: &mut Checker) {
    // TODO: Add rules for ClassVariableTargetNode
}
