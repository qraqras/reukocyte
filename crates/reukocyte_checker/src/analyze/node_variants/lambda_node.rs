#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::LambdaNode;

/// Run lint rules over a [`LambdaNode`] syntax node.
pub(crate) fn lambda_node(node: &LambdaNode, checker: &mut Checker) {
    // TODO: Add rules for LambdaNode
}
