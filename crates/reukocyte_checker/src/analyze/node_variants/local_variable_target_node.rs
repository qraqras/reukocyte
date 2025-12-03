#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::LocalVariableTargetNode;

/// Run lint rules over a [`LocalVariableTargetNode`] syntax node.
pub(crate) fn local_variable_target_node(node: &LocalVariableTargetNode, checker: &mut Checker) {
    // TODO: Add rules for LocalVariableTargetNode
}
