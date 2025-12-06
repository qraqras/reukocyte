use ruby_prism::Node;

/// Returns the first part of a method call chain.
///
/// For example, given `foo.bar.baz`, this returns `foo`.
/// For `foo.bar { block }.baz.qux`, this also returns `foo`.
///
/// In Prism's AST, blocks are attached to CallNode as a `block` field,
/// not as a separate node in the receiver chain. So we only need to
/// traverse the `receiver` of CallNodes.
///
/// This is equivalent to RuboCop's `first_part_of_call_chain` in `lib/rubocop/cop/util.rb`.
pub fn first_part_of_call_chain(node: Node) -> Option<Node> {
    let mut current = Some(node);
    while let Some(node) = &current {
        if let Some(call_node) = node.as_call_node() {
            current = call_node.receiver();
            continue;
        }
        break;
    }
    current
}
