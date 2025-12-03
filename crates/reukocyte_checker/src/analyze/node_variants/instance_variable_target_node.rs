#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InstanceVariableTargetNode;

/// Run lint rules over a [`InstanceVariableTargetNode`] syntax node.
pub(crate) fn instance_variable_target_node(node: &InstanceVariableTargetNode, checker: &mut Checker) {
    // TODO: Add rules for InstanceVariableTargetNode
}
