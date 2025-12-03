#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::NumberedParametersNode;

/// Run lint rules over a [`NumberedParametersNode`] syntax node.
pub(crate) fn numbered_parameters_node(node: &NumberedParametersNode, checker: &mut Checker) {
    // TODO: Add rules for NumberedParametersNode
}
