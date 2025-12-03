#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InstanceVariableReadNode;

/// Run lint rules over a [`InstanceVariableReadNode`] syntax node.
pub(crate) fn instance_variable_read_node(node: &InstanceVariableReadNode, checker: &mut Checker) {
    // TODO: Add rules for InstanceVariableReadNode
}
