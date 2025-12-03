#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::LocalVariableReadNode;

/// Run lint rules over a [`LocalVariableReadNode`] syntax node.
pub(crate) fn local_variable_read_node(node: &LocalVariableReadNode, checker: &mut Checker) {
    // TODO: Add rules for LocalVariableReadNode
}
