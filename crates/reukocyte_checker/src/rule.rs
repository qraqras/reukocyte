use std::fmt;

use crate::checker::Checker;

/// Unique identifier for a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RuleId {
    Layout(LayoutRule),
    Lint(LintRule),
}
impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.category().as_str(), self.name())
    }
}

impl RuleId {
    /// Get the category of the rule.
    pub const fn category(&self) -> Category {
        match self {
            Self::Layout(_) => Category::Layout,
            Self::Lint(_) => Category::Lint,
        }
    }
    /// Get the rule name without category.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Layout(rule) => rule.name(),
            Self::Lint(rule) => rule.name(),
        }
    }

    /// Rules that this rule's autocorrection conflicts with.
    ///
    /// When two rules conflict, only one of them should have its
    /// fixes applied in a single iteration. The skipped rule's fixes will
    /// be applied in a subsequent iteration.
    ///
    /// This is equivalent to RuboCop's `autocorrect_incompatible_with`.
    pub fn conflicts_with(&self) -> &'static [RuleId] {
        match self {
            Self::Layout(LayoutRule::BeginEndAlignment) => &[],
            Self::Layout(LayoutRule::DefEndAlignment) => &[],
            Self::Layout(LayoutRule::EmptyLines) => &[],
            Self::Layout(LayoutRule::EndAlignment) => &[],
            Self::Layout(LayoutRule::IndentationConsistency) => &[],
            Self::Layout(LayoutRule::IndentationStyle) => &[],
            Self::Layout(LayoutRule::IndentationWidth) => &[],
            Self::Layout(LayoutRule::LeadingEmptyLines) => &[],
            Self::Layout(LayoutRule::TrailingEmptyLines) => &[],
            Self::Layout(LayoutRule::TrailingWhitespace) => &[],
            Self::Lint(LintRule::Debugger) => &[],
        }
    }

    /// Check if this rule conflicts with another rule.
    pub fn has_conflict_with(&self, other: RuleId) -> bool {
        self.conflicts_with().contains(&other)
    }
}

/// Category of a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Category {
    Layout,
    Lint,
}
impl Category {
    /// Get the category name as a string.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Layout => "Layout",
            Self::Lint => "Lint",
        }
    }
}

/// Layout rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LayoutRule {
    BeginEndAlignment,
    DefEndAlignment,
    EmptyLines,
    EndAlignment,
    IndentationConsistency,
    IndentationStyle,
    IndentationWidth,
    LeadingEmptyLines,
    TrailingEmptyLines,
    TrailingWhitespace,
}
impl LayoutRule {
    /// Get the rule name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::BeginEndAlignment => "BeginEndAlignment",
            Self::DefEndAlignment => "DefEndAlignment",
            Self::EmptyLines => "EmptyLines",
            Self::EndAlignment => "EndAlignment",
            Self::IndentationConsistency => "IndentationConsistency",
            Self::IndentationStyle => "IndentationStyle",
            Self::IndentationWidth => "IndentationWidth",
            Self::LeadingEmptyLines => "LeadingEmptyLines",
            Self::TrailingEmptyLines => "TrailingEmptyLines",
            Self::TrailingWhitespace => "TrailingWhitespace",
        }
    }
}

/// Lint rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LintRule {
    Debugger,
}
impl LintRule {
    /// Get the rule name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Debugger => "Debugger",
        }
    }
}

// ============================================================================
// Rule Traits
// ============================================================================

/// Base trait for all rules.
///
/// Each rule struct should implement this trait to provide its identifier.
pub trait Rule {
    /// The unique identifier for this rule.
    const ID: RuleId;
}

/// Trait for rules that check a specific node type.
///
/// Rules implement this trait for each node type they want to check.
///
/// # Example
/// ```ignore
/// impl Check<StatementsNode> for IndentationWidth {
///     fn check(node: &StatementsNode, checker: &mut Checker) {
///         // check logic
///     }
/// }
/// ```
pub trait Check<N>: Rule {
    fn check(node: &N, checker: &mut Checker);
}

// ============================================================================
// Rule Runner Macro
// ============================================================================

/// Macro to run multiple rules for a specific node type.
///
/// This macro generates static dispatch calls for each rule, avoiding
/// the overhead of dynamic dispatch (trait objects).
///
/// # Example
/// ```ignore
/// run_rules!(node, checker, StatementsNode, [
///     EndAlignment,
///     IndentationWidth,
/// ]);
/// ```
#[macro_export]
macro_rules! run_rules {
    ($node:expr, $checker:expr, $node_type:ty, [$($rule:ty),* $(,)?]) => {
        $(
            if $checker.is_enabled(<$rule as $crate::rule::Rule>::ID) {
                <$rule as $crate::rule::Check<$node_type>>::check($node, $checker);
            }
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_id_parts() {
        let layout_rule = RuleId::Layout(LayoutRule::TrailingWhitespace);
        let lint_rule = RuleId::Lint(LintRule::Debugger);

        assert_eq!(layout_rule.category(), Category::Layout);
        assert_eq!(layout_rule.name(), "TrailingWhitespace");
        assert_eq!(lint_rule.category(), Category::Lint);
        assert_eq!(lint_rule.name(), "Debugger");
    }

    #[test]
    fn test_rule_id_display() {
        assert_eq!(
            format!(
                "{}/{}",
                RuleId::Layout(LayoutRule::TrailingWhitespace).category().as_str(),
                RuleId::Layout(LayoutRule::TrailingWhitespace).name()
            ),
            "Layout/TrailingWhitespace"
        );
        assert_eq!(
            format!(
                "{}/{}",
                RuleId::Lint(LintRule::Debugger).category().as_str(),
                RuleId::Lint(LintRule::Debugger).name()
            ),
            "Lint/Debugger"
        );
    }

    #[test]
    fn test_no_conflict() {
        let rule = RuleId::Layout(LayoutRule::TrailingWhitespace);
        assert!(!rule.has_conflict_with(RuleId::Lint(LintRule::Debugger)));
    }

    #[test]
    fn test_rule_id_equality() {
        assert_eq!(RuleId::Layout(LayoutRule::TrailingWhitespace), RuleId::Layout(LayoutRule::TrailingWhitespace));
        assert_ne!(RuleId::Layout(LayoutRule::TrailingWhitespace), RuleId::Lint(LintRule::Debugger));
    }
}
