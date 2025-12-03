#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::SymbolNode;

/// Run lint rules over a [`SymbolNode`] syntax node.
pub(crate) fn symbol_node(node: &SymbolNode, checker: &mut Checker) {
    // TODO: Add rules for SymbolNode
}
