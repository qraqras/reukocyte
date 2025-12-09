use crate::conflict::ConflictRegistry;
use crate::corrector;
use crate::corrector::Corrector;
use crate::rule::RuleId;
use crate::{Diagnostic, check};
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Maximum number of iterations to prevent infinite loops.
/// RuboCop uses 200 as well.
const MAX_ITERATIONS: usize = 200;

/// Error indicating that the autocorrection loop got stuck.
#[derive(Debug, Clone)]
pub struct InfiniteCorrectionLoop {
    pub path: Option<String>,         // Optional file path
    pub iteration: usize,             // Current iteration count
    pub loop_start: Option<usize>,    // Where the loop started, if known
    pub offending_rules: Vec<String>, // Rules that caused the loop
}

impl InfiniteCorrectionLoop {
    /// Create a new InfiniteCorrectionLoop error.
    fn new(
        path: Option<&str>,
        iteration: usize,
        loop_start: Option<usize>,
        rules_by_iteration: &[HashSet<RuleId>],
    ) -> Self {
        // Extract the offending rules from the loop iterations
        let offending_rules = if let Some(start) = loop_start {
            rules_by_iteration[start..]
                .iter()
                .flat_map(|rules| rules.iter().map(|r| r.to_string()))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        // Deduplicate while preserving order
        let mut seen = HashSet::new();
        let offending_rules: Vec<String> = offending_rules
            .into_iter()
            .filter(|r| seen.insert(r.clone()))
            .collect();
        Self {
            path: path.map(|s| s.to_string()),
            iteration,
            loop_start,
            offending_rules,
        }
    }
}

impl std::fmt::Display for InfiniteCorrectionLoop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Infinite loop detected")?;

        if let Some(path) = &self.path {
            write!(f, " in {}", path)?;
        }

        if !self.offending_rules.is_empty() {
            write!(f, " and caused by {}", self.offending_rules.join(" -> "))?;
        } else if let Some(start) = self.loop_start {
            write!(
                f,
                ": iteration {} produced the same source as iteration {}",
                self.iteration, start
            )?;
        } else {
            write!(f, ": exceeded {} iterations", MAX_ITERATIONS)?;
        }

        Ok(())
    }
}
impl std::error::Error for InfiniteCorrectionLoop {}

/// Calculate a checksum for the source code.
fn checksum(source: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

/// Apply all fixes from diagnostics to the source code.
///
/// Uses RuboCop-style iterative correction with conflict detection:
/// 1. Create a Corrector and ConflictRegistry for each iteration
/// 2. Skip fixes from rules that conflict with already-applied rules
/// 3. Skip fixes that have edit-level conflicts
/// 4. Apply merged fixes in one pass
/// 5. Re-check the source to find remaining violations
/// 6. Repeat until no more fixes are applied or max iterations reached
///
/// # Arguments
///
/// * `path` - Optional file path for error reporting (RuboCop compatible)
/// * `source` - The original source code
/// * `diagnostics` - The initial diagnostics with fixes to apply
/// * `unsafe_fixes` - Whether to apply unsafe fixes
///
/// # Returns
///
/// A tuple of (corrected source, total number of fixes applied)
pub fn apply_fixes(
    path: Option<&str>,
    source: &[u8],
    diagnostics: &[Diagnostic],
    unsafe_fixes: bool,
) -> (Vec<u8>, usize) {
    match apply_fixes_with_loop_detection(path, source, diagnostics, unsafe_fixes) {
        Ok((source, count)) => (source, count),
        Err(err) => {
            // Log the error but return what we have
            eprintln!("Warning: {}", err);
            (source.to_vec(), 0)
        }
    }
}

/// Apply fixes with infinite loop detection.
///
/// Returns an error if an infinite loop is detected.
/// The error includes the file path and the rules that caused the loop,
/// matching RuboCop's `InfiniteCorrectionLoop` behavior.
pub fn apply_fixes_with_loop_detection(
    path: Option<&str>,
    source: &[u8],
    diagnostics: &[Diagnostic],
    unsafe_fixes: bool,
) -> Result<(Vec<u8>, usize), InfiniteCorrectionLoop> {
    let mut current_source = source.to_vec();
    let mut total_fixed = 0;
    let mut current_diagnostics = diagnostics.to_vec();

    // Track checksums to detect loops (A -> B -> A pattern)
    let mut seen_checksums: Vec<u64> = Vec::new();

    // Track which rules were applied in each iteration (for error reporting)
    let mut rules_by_iteration: Vec<HashSet<RuleId>> = Vec::new();

    for iteration in 0..MAX_ITERATIONS {
        // Check for infinite loop via checksum
        let current_checksum = checksum(&current_source);
        if let Some(loop_start) = seen_checksums.iter().position(|&c| c == current_checksum) {
            return Err(InfiniteCorrectionLoop::new(
                path,
                iteration,
                Some(loop_start),
                &rules_by_iteration,
            ));
        }
        seen_checksums.push(current_checksum);

        // Create a new Corrector and ConflictRegistry for this iteration
        let mut corrector = Corrector::new();
        let mut conflict_registry = ConflictRegistry::new();
        let mut applied_rules_this_iteration: HashSet<RuleId> = HashSet::new();

        // Try to merge all applicable fixes
        for diagnostic in &current_diagnostics {
            if let Some(fix) = &diagnostic.fix {
                if !corrector::should_apply_fix(fix, unsafe_fixes) {
                    continue;
                }
                // Check for rule-level conflicts
                if conflict_registry.conflicts_with_applied(diagnostic.rule_id) {
                    // This rule conflicts with an already-applied rule
                    // Skip it and retry in the next iteration
                    continue;
                }
                // Try to merge at edit level; if it conflicts, skip it
                if corrector.merge(fix).is_ok() {
                    // Mark this rule as applied for conflict checking
                    conflict_registry.mark_applied(diagnostic.rule_id);
                    applied_rules_this_iteration.insert(diagnostic.rule_id);
                }
            }
        }

        // Record which rules were applied this iteration
        rules_by_iteration.push(applied_rules_this_iteration);

        if corrector.is_empty() {
            // No more fixes to apply
            break;
        }

        // Update total fixed count
        total_fixed += corrector.edit_count();

        // Apply the merged edits
        current_source = corrector.apply(&current_source);

        // Re-check to find remaining violations (RuboCop style)
        current_diagnostics = check(&current_source);

        if current_diagnostics.iter().all(|d| d.fix.is_none()) {
            // No more fixable diagnostics
            break;
        }
    }

    // If we exhausted iterations, that's also a loop
    if seen_checksums.len() >= MAX_ITERATIONS {
        return Err(InfiniteCorrectionLoop::new(
            path,
            MAX_ITERATIONS,
            None,
            &rules_by_iteration,
        ));
    }
    Ok((current_source, total_fixed))
}

/// Apply fixes and return the result along with remaining diagnostics.
pub fn apply_fixes_with_remaining(
    path: Option<&str>,
    source: &[u8],
    diagnostics: &[Diagnostic],
    unsafe_fixes: bool,
) -> (Vec<u8>, Vec<Diagnostic>, usize) {
    let (fixed_source, fix_count) = apply_fixes(path, source, diagnostics, unsafe_fixes);
    // Re-check to get remaining diagnostics
    let remaining = check(&fixed_source);
    (fixed_source, remaining, fix_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;

    #[test]
    fn test_apply_trailing_whitespace_fix() {
        let source = b"def foo  \n  bar\nend\n";
        let diagnostics = check(source);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].fix.is_some());

        let (fixed, count) = apply_fixes(None, source, &diagnostics, false);

        assert_eq!(count, 1);
        assert_eq!(fixed, b"def foo\n  bar\nend\n");
    }

    #[test]
    fn test_apply_multiple_fixes() {
        let source = b"def foo  \n  bar  \nend\n";
        let diagnostics = check(source);

        assert_eq!(diagnostics.len(), 2);

        let (fixed, count) = apply_fixes(None, source, &diagnostics, false);

        assert_eq!(count, 2);
        assert_eq!(fixed, b"def foo\n  bar\nend\n");
    }

    #[test]
    fn test_no_fix_for_debugger() {
        let source = b"def foo\n  binding.pry\nend\n";
        let diagnostics = check(source);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].fix.is_none());

        let (fixed, count) = apply_fixes(None, source, &diagnostics, false);

        assert_eq!(count, 0);
        assert_eq!(fixed, source);
    }

    #[test]
    fn test_mixed_fixes() {
        // Source with both trailing whitespace (fixable) and debugger (not fixable)
        let source = b"def foo  \n  binding.pry\nend\n";
        let diagnostics = check(source);

        assert_eq!(diagnostics.len(), 2);

        let (fixed, count) = apply_fixes(None, source, &diagnostics, false);

        assert_eq!(count, 1); // Only trailing whitespace fixed
        assert_eq!(fixed, b"def foo\n  binding.pry\nend\n");
    }

    #[test]
    fn test_apply_fixes_with_remaining() {
        let source = b"def foo  \n  binding.pry\nend\n";
        let diagnostics = check(source);

        let (fixed, remaining, count) =
            apply_fixes_with_remaining(None, source, &diagnostics, false);

        assert_eq!(count, 1);
        assert_eq!(fixed, b"def foo\n  binding.pry\nend\n");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].rule(), "Lint/Debugger");
    }

    #[test]
    fn test_whitespace_only_line() {
        let source = b"def foo\n   \nend\n";
        let diagnostics = check(source);

        assert_eq!(diagnostics.len(), 1);

        let (fixed, count) = apply_fixes(None, source, &diagnostics, false);

        assert_eq!(count, 1);
        assert_eq!(fixed, b"def foo\n\nend\n");
    }
}

/// Tests for rule conflict handling.
/// These tests verify the ConflictRegistry integration.
#[cfg(test)]
mod conflict_tests {
    use crate::conflict::ConflictRegistry;
    use crate::rule::{LayoutRule, LintRule, RuleId};

    const RULE_WHITESPACE: RuleId = RuleId::Layout(LayoutRule::TrailingWhitespace);
    const RULE_DEBUGGER: RuleId = RuleId::Lint(LintRule::Debugger);

    #[test]
    fn test_rule_conflict_skipping() {
        // Scenario: If TrailingWhitespace declared Debugger as conflicting,
        // when TrailingWhitespace is applied first, Debugger should be skipped.
        // Note: Currently neither rule declares conflicts, so this test just
        // verifies the mechanism works with the registry.
        let mut registry = ConflictRegistry::new();

        // TrailingWhitespace applied first
        registry.mark_applied(RULE_WHITESPACE);

        // Since TrailingWhitespace doesn't declare Debugger as conflicting,
        // Debugger should NOT be skipped
        assert!(
            !registry.conflicts_with_applied(RULE_DEBUGGER),
            "Debugger should not be skipped because no conflicts are declared"
        );
    }

    #[test]
    fn test_reverse_conflict_skipping() {
        // Scenario: If conflicts were declared, they would be bidirectional
        // Note: Currently neither rule declares conflicts
        let mut registry = ConflictRegistry::new();

        // Debugger applied first
        registry.mark_applied(RULE_DEBUGGER);

        // Since no conflicts are declared, TrailingWhitespace should not be skipped
        assert!(
            !registry.conflicts_with_applied(RULE_WHITESPACE),
            "TrailingWhitespace should not be skipped because no conflicts are declared"
        );
    }

    #[test]
    fn test_conflict_resolution_in_next_iteration() {
        // Scenario: After clearing the registry (new iteration),
        // previously skipped rules can be applied
        let mut registry = ConflictRegistry::new();

        // First iteration: apply TrailingWhitespace
        registry.mark_applied(RULE_WHITESPACE);

        // (If conflicts existed, Debugger would be skipped here)
        // After clearing, any skipped rules can be applied
        registry.clear();

        // Now Debugger can definitely be applied
        assert!(!registry.conflicts_with_applied(RULE_DEBUGGER));
    }
}

/// Tests for infinite loop detection.
#[cfg(test)]
mod loop_detection_tests {
    use super::*;

    #[test]
    fn test_checksum_different_sources() {
        let source1 = b"hello";
        let source2 = b"world";
        assert_ne!(checksum(source1), checksum(source2));
    }

    #[test]
    fn test_checksum_same_source() {
        let source = b"hello world";
        assert_eq!(checksum(source), checksum(source));
    }

    #[test]
    fn test_infinite_loop_error_display_with_path() {
        let err = InfiniteCorrectionLoop {
            path: Some("example.rb".to_string()),
            iteration: 5,
            loop_start: Some(2),
            offending_rules: vec![
                "Layout/TrailingWhitespace".to_string(),
                "Lint/Debugger".to_string(),
            ],
        };
        let msg = err.to_string();
        assert!(msg.contains("example.rb"));
        assert!(msg.contains("Layout/TrailingWhitespace -> Lint/Debugger"));
    }

    #[test]
    fn test_infinite_loop_error_display_without_path() {
        let err = InfiniteCorrectionLoop {
            path: None,
            iteration: 5,
            loop_start: Some(2),
            offending_rules: vec![],
        };
        let msg = err.to_string();
        assert!(msg.contains("Infinite loop detected"));
        assert!(msg.contains("iteration 5"));
        assert!(msg.contains("iteration 2"));
    }

    #[test]
    fn test_infinite_loop_error_display_max_iterations() {
        let err = InfiniteCorrectionLoop {
            path: Some("test.rb".to_string()),
            iteration: 200,
            loop_start: None,
            offending_rules: vec![],
        };
        let msg = err.to_string();
        assert!(msg.contains("test.rb"));
        assert!(msg.contains("200"));
    }

    #[test]
    fn test_no_loop_on_normal_fix() {
        let source = b"def foo  \nend\n";
        let diagnostics = check(source);

        let result = apply_fixes_with_loop_detection(Some("test.rb"), source, &diagnostics, false);
        assert!(result.is_ok());

        let (fixed, count) = result.unwrap();
        assert_eq!(count, 1);
        assert_eq!(fixed, b"def foo\nend\n");
    }
}
