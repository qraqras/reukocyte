#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InterpolatedSymbolNode;

/// Run lint rules over a [`InterpolatedSymbolNode`] syntax node.
pub(crate) fn interpolated_symbol_node(node: &InterpolatedSymbolNode, checker: &mut Checker) {
    // TODO: Add rules for InterpolatedSymbolNode
}
