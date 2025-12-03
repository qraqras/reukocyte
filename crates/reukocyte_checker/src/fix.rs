//! Fix application utilities.
//!
//! This module provides functions to apply fixes from diagnostics to source code.
//! Uses RuboCop-style iterative correction: applies fixes in multiple passes
//! until no more changes are made.

use crate::{Applicability, Diagnostic, Edit, check};

/// Maximum number of iterations to prevent infinite loops.
const MAX_ITERATIONS: usize = 10;

/// Apply all fixes from diagnostics to the source code.
///
/// Uses RuboCop-style iterative correction:
/// 1. Apply non-overlapping fixes in one pass
/// 2. Re-check the source to find remaining violations
/// 3. Repeat until no more fixes are applied or max iterations reached
///
/// # Arguments
///
/// * `source` - The original source code
/// * `diagnostics` - The initial diagnostics with fixes to apply
/// * `unsafe_fixes` - Whether to apply unsafe fixes
///
/// # Returns
///
/// A tuple of (corrected source, total number of fixes applied)
pub fn apply_fixes(
    source: &[u8],
    diagnostics: &[Diagnostic],
    unsafe_fixes: bool,
) -> (Vec<u8>, usize) {
    let mut current_source = source.to_vec();
    let mut total_fixed = 0;
    let mut current_diagnostics = diagnostics.to_vec();

    for _iteration in 0..MAX_ITERATIONS {
        let (new_source, fix_count) =
            apply_fixes_single_pass(&current_source, &current_diagnostics, unsafe_fixes);

        if fix_count == 0 {
            // No more fixes applied, we're done
            break;
        }

        total_fixed += fix_count;
        current_source = new_source;

        // Re-check to find remaining violations (RuboCop style)
        current_diagnostics = check(&current_source);

        if current_diagnostics.iter().all(|d| d.fix.is_none()) {
            // No more fixable diagnostics
            break;
        }
    }

    (current_source, total_fixed)
}

/// Apply fixes in a single pass (non-overlapping only).
///
/// This is the internal function that applies fixes once.
/// Overlapping fixes are skipped and will be handled in the next iteration.
fn apply_fixes_single_pass(
    source: &[u8],
    diagnostics: &[Diagnostic],
    unsafe_fixes: bool,
) -> (Vec<u8>, usize) {
    // Collect all edits that should be applied
    let mut edits: Vec<&Edit> = Vec::new();

    for diagnostic in diagnostics {
        if let Some(fix) = &diagnostic.fix {
            let should_apply = match fix.applicability {
                Applicability::Safe => true,
                Applicability::Unsafe => unsafe_fixes,
                Applicability::DisplayOnly => false,
            };

            if should_apply {
                edits.extend(fix.edits.iter());
            }
        }
    }

    if edits.is_empty() {
        return (source.to_vec(), 0);
    }

    // Sort edits by start position (ascending) for forward construction
    edits.sort_by_key(|e| (e.start, e.end));

    // Build result by copying unchanged parts and applying edits
    // This is more efficient than splice and matches Ruff's approach
    let mut result = Vec::with_capacity(source.len());
    let mut last_pos = 0;
    let mut fix_count = 0;

    for edit in edits {
        // Skip overlapping edits (will be handled in next iteration)
        if edit.start < last_pos {
            continue;
        }

        // Copy unchanged content before this edit
        result.extend_from_slice(&source[last_pos..edit.start]);

        // Apply the edit
        result.extend_from_slice(edit.content.as_bytes());

        last_pos = edit.end;
        fix_count += 1;
    }

    // Copy remaining content after last edit
    result.extend_from_slice(&source[last_pos..]);

    (result, fix_count)
}

/// Apply fixes and return the result along with remaining diagnostics.
///
/// This is useful for getting both the fixed source and unfixable diagnostics.
pub fn apply_fixes_with_remaining(
    source: &[u8],
    diagnostics: &[Diagnostic],
    unsafe_fixes: bool,
) -> (Vec<u8>, Vec<Diagnostic>, usize) {
    let (fixed_source, fix_count) = apply_fixes(source, diagnostics, unsafe_fixes);

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

        let (fixed, count) = apply_fixes(source, &diagnostics, false);

        assert_eq!(count, 1);
        assert_eq!(fixed, b"def foo\n  bar\nend\n");
    }

    #[test]
    fn test_apply_multiple_fixes() {
        let source = b"def foo  \n  bar  \nend\n";
        let diagnostics = check(source);

        assert_eq!(diagnostics.len(), 2);

        let (fixed, count) = apply_fixes(source, &diagnostics, false);

        assert_eq!(count, 2);
        assert_eq!(fixed, b"def foo\n  bar\nend\n");
    }

    #[test]
    fn test_no_fix_for_debugger() {
        let source = b"def foo\n  binding.pry\nend\n";
        let diagnostics = check(source);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].fix.is_none());

        let (fixed, count) = apply_fixes(source, &diagnostics, false);

        assert_eq!(count, 0);
        assert_eq!(fixed, source);
    }

    #[test]
    fn test_mixed_fixes() {
        // Source with both trailing whitespace (fixable) and debugger (not fixable)
        let source = b"def foo  \n  binding.pry\nend\n";
        let diagnostics = check(source);

        assert_eq!(diagnostics.len(), 2);

        let (fixed, count) = apply_fixes(source, &diagnostics, false);

        assert_eq!(count, 1); // Only trailing whitespace fixed
        assert_eq!(fixed, b"def foo\n  binding.pry\nend\n");
    }

    #[test]
    fn test_apply_fixes_with_remaining() {
        let source = b"def foo  \n  binding.pry\nend\n";
        let diagnostics = check(source);

        let (fixed, remaining, count) = apply_fixes_with_remaining(source, &diagnostics, false);

        assert_eq!(count, 1);
        assert_eq!(fixed, b"def foo\n  binding.pry\nend\n");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].rule, "Lint/Debugger");
    }

    #[test]
    fn test_whitespace_only_line() {
        let source = b"def foo\n   \nend\n";
        let diagnostics = check(source);

        assert_eq!(diagnostics.len(), 1);

        let (fixed, count) = apply_fixes(source, &diagnostics, false);

        assert_eq!(count, 1);
        assert_eq!(fixed, b"def foo\n\nend\n");
    }
}
