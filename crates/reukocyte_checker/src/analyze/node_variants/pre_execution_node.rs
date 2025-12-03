#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::PreExecutionNode;

/// Run lint rules over a [`PreExecutionNode`] syntax node.
pub(crate) fn pre_execution_node(node: &PreExecutionNode, checker: &mut Checker) {
    // TODO: Add rules for PreExecutionNode
}
