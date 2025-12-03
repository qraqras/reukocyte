use crate::rule::RuleId;
use std::collections::HashSet;

/// Registry to track which rules have had their fixes applied in the current iteration.
///
/// This implements RuboCop's `autocorrect_incompatible_with` behavior:
/// - When a rule's fixes are applied, it's marked as "applied"
/// - When checking if a new rule can apply, we check if it conflicts
///   with any already-applied rules
/// - Conflicting rules are skipped and retried in the next iteration
#[derive(Debug, Default)]
pub struct ConflictRegistry {
    applied_rules: HashSet<RuleId>,
}
impl ConflictRegistry {
    /// Create a new empty conflict registry.
    pub fn new() -> Self {
        Self {
            applied_rules: HashSet::new(),
        }
    }
    /// Mark a rule as having had its fixes applied.
    pub fn mark_applied(&mut self, rule_id: RuleId) {
        self.applied_rules.insert(rule_id);
    }
    /// Check if a rule was already applied in this iteration.
    pub fn was_applied(&self, rule_id: RuleId) -> bool {
        self.applied_rules.contains(&rule_id)
    }
    /// Check if a rule has conflicts with already-applied rules.
    ///
    /// A rule has conflicts if:
    /// 1. Any already-applied rule declares it as conflicting, OR
    /// 2. The rule itself declares any already-applied rule as conflicting
    ///
    /// This bidirectional check ensures that conflicts are symmetric:
    /// if A conflicts with B, then both A and B should not be
    /// applied in the same iteration (whichever comes first wins).
    pub fn conflicts_with_applied(&self, rule_id: RuleId) -> bool {
        // Check if this rule declares any applied rule as conflicting
        for &conflicting_rule_id in rule_id.conflicts_with() {
            if self.applied_rules.contains(&conflicting_rule_id) {
                return true;
            }
        }
        // Check if any applied rule declares this rule as conflicting
        for applied_rule_id in &self.applied_rules {
            if applied_rule_id.has_conflict_with(rule_id) {
                return true;
            }
        }
        false
    }
    /// Clear the registry for a new iteration.
    pub fn clear(&mut self) {
        self.applied_rules.clear();
    }
    /// Get the number of rules applied in this iteration.
    pub fn applied_count(&self) -> usize {
        self.applied_rules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{LayoutRule, LintRule};

    // Use actual rule IDs for testing
    const RULE_A: RuleId = RuleId::Layout(LayoutRule::TrailingWhitespace);
    const RULE_B: RuleId = RuleId::Lint(LintRule::Debugger);

    #[test]
    fn test_empty_registry() {
        let registry = ConflictRegistry::new();

        // Nothing applied yet, so nothing should be skipped
        assert!(!registry.conflicts_with_applied(RULE_A));
        assert!(!registry.conflicts_with_applied(RULE_B));
    }

    #[test]
    fn test_forward_conflict() {
        // If A declares B as conflicting and A is applied first, B should be skipped
        // Note: Currently no rules have conflicts defined, so this test
        // just verifies the mechanism works
        let mut registry = ConflictRegistry::new();

        registry.mark_applied(RULE_A);

        // Since RULE_A doesn't declare RULE_B as conflicting, B has no conflict
        assert!(!registry.conflicts_with_applied(RULE_B));
    }

    #[test]
    fn test_clear_registry() {
        let mut registry = ConflictRegistry::new();

        registry.mark_applied(RULE_A);
        assert!(registry.was_applied(RULE_A));

        registry.clear();
        assert!(!registry.was_applied(RULE_A));
    }

    #[test]
    fn test_was_applied() {
        let mut registry = ConflictRegistry::new();

        assert!(!registry.was_applied(RULE_A));
        registry.mark_applied(RULE_A);
        assert!(registry.was_applied(RULE_A));
        assert!(!registry.was_applied(RULE_B));
    }
}
