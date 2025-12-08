use ruby_prism::*;

/// A wrapper enum for all conditional node types.
///
/// This allows rules to implement `Check<ConditionalNode<'_>>` once
/// instead of implementing for each individual conditional type.
#[derive(Debug, Clone, Copy)]
pub enum ConditionalNode<'a> {
    CaseMatchNode(&'a CaseMatchNode<'a>),
    CaseNode(&'a CaseNode<'a>),
    IfNode(&'a IfNode<'a>),
    UnlessNode(&'a UnlessNode<'a>),
    UntilNode(&'a UntilNode<'a>),
    WhileNode(&'a WhileNode<'a>),
}

impl<'a> ConditionalNode<'a> {
    /// Get the underlying node as a generic `Node`.
    pub fn as_node(&self) -> Node<'a> {
        match self {
            Self::CaseMatchNode(n) => n.as_node(),
            Self::CaseNode(n) => n.as_node(),
            Self::IfNode(n) => n.as_node(),
            Self::UnlessNode(n) => n.as_node(),
            Self::UntilNode(n) => n.as_node(),
            Self::WhileNode(n) => n.as_node(),
        }
    }

    /// Get the location of the entire assignment node.
    pub fn location(&self) -> Location<'a> {
        match self {
            Self::CaseMatchNode(n) => n.location(),
            Self::CaseNode(n) => n.location(),
            Self::IfNode(n) => n.location(),
            Self::UnlessNode(n) => n.location(),
            Self::UntilNode(n) => n.location(),
            Self::WhileNode(n) => n.location(),
        }
    }

    /// Get the location of the conditional keyword (e.g., `if`, `case`).
    pub fn keyword_loc(&self) -> Option<Location<'a>> {
        match self {
            Self::CaseMatchNode(n) => Some(n.case_keyword_loc()),
            Self::CaseNode(n) => Some(n.case_keyword_loc()),
            Self::IfNode(n) => n.if_keyword_loc(),
            Self::UnlessNode(n) => Some(n.keyword_loc()),
            Self::UntilNode(n) => Some(n.keyword_loc()),
            Self::WhileNode(n) => Some(n.keyword_loc()),
        }
    }

    pub fn end_keyword_loc(&self) -> Option<Location<'a>> {
        match self {
            Self::CaseMatchNode(n) => Some(n.end_keyword_loc()),
            Self::CaseNode(n) => Some(n.end_keyword_loc()),
            Self::IfNode(n) => n.end_keyword_loc(),
            Self::UnlessNode(n) => n.end_keyword_loc(),
            Self::UntilNode(n) => n.closing_loc(),
            Self::WhileNode(n) => n.closing_loc(),
        }
    }
}

// ============================================================================
// From implementations for easy conversion
// ============================================================================

impl<'a> From<&'a CaseMatchNode<'a>> for ConditionalNode<'a> {
    fn from(node: &'a CaseMatchNode<'a>) -> Self {
        Self::CaseMatchNode(node)
    }
}
impl<'a> From<&'a CaseNode<'a>> for ConditionalNode<'a> {
    fn from(node: &'a CaseNode<'a>) -> Self {
        Self::CaseNode(node)
    }
}
impl<'a> From<&'a IfNode<'a>> for ConditionalNode<'a> {
    fn from(node: &'a IfNode<'a>) -> Self {
        Self::IfNode(node)
    }
}
impl<'a> From<&'a UnlessNode<'a>> for ConditionalNode<'a> {
    fn from(node: &'a UnlessNode<'a>) -> Self {
        Self::UnlessNode(node)
    }
}
impl<'a> From<&'a UntilNode<'a>> for ConditionalNode<'a> {
    fn from(node: &'a UntilNode<'a>) -> Self {
        Self::UntilNode(node)
    }
}
impl<'a> From<&'a WhileNode<'a>> for ConditionalNode<'a> {
    fn from(node: &'a WhileNode<'a>) -> Self {
        Self::WhileNode(node)
    }
}
