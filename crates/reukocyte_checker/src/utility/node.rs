use ruby_prism::Node;

pub fn is_assignment(node: &Node) -> bool {
    match node {
        Node::CallAndWriteNode { .. } => true,
        Node::CallOperatorWriteNode { .. } => true,
        Node::CallOrWriteNode { .. } => true,
        Node::ClassVariableAndWriteNode { .. } => true,
        Node::ClassVariableOperatorWriteNode { .. } => true,
        Node::ClassVariableOrWriteNode { .. } => true,
        Node::ClassVariableWriteNode { .. } => true,
        Node::ConstantAndWriteNode { .. } => true,
        Node::ConstantOperatorWriteNode { .. } => true,
        Node::ConstantOrWriteNode { .. } => true,
        Node::ConstantPathAndWriteNode { .. } => true,
        Node::ConstantPathOperatorWriteNode { .. } => true,
        Node::ConstantPathOrWriteNode { .. } => true,
        Node::ConstantPathWriteNode { .. } => true,
        Node::ConstantWriteNode { .. } => true,
        Node::GlobalVariableAndWriteNode { .. } => true,
        Node::GlobalVariableOperatorWriteNode { .. } => true,
        Node::GlobalVariableOrWriteNode { .. } => true,
        Node::GlobalVariableWriteNode { .. } => true,
        Node::IndexAndWriteNode { .. } => true,
        Node::IndexOperatorWriteNode { .. } => true,
        Node::IndexOrWriteNode { .. } => true,
        Node::InstanceVariableAndWriteNode { .. } => true,
        Node::InstanceVariableOperatorWriteNode { .. } => true,
        Node::InstanceVariableOrWriteNode { .. } => true,
        Node::InstanceVariableWriteNode { .. } => true,
        Node::LocalVariableAndWriteNode { .. } => true,
        Node::LocalVariableOperatorWriteNode { .. } => true,
        Node::LocalVariableOrWriteNode { .. } => true,
        Node::LocalVariableWriteNode { .. } => true,
        Node::MatchWriteNode { .. } => true,
        Node::MultiWriteNode { .. } => true,
        _ => false,
    }
}

pub fn is_conditional(node: &Node) -> bool {
    match node {
        Node::CaseMatchNode { .. } => true,
        Node::CaseNode { .. } => true,
        Node::IfNode { .. } => true,
        Node::UnlessNode { .. } => true,
        Node::UntilNode { .. } => true,
        Node::WhileNode { .. } => true,
        _ => false,
    }
}
