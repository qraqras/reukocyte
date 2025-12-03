#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ItParametersNode;

/// Run lint rules over a [`ItParametersNode`] syntax node.
pub(crate) fn it_parameters_node(node: &ItParametersNode, checker: &mut Checker) {
    // TODO: Add rules for ItParametersNode
}
