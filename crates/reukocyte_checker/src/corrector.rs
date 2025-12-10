use crate::{Applicability, Edit, Fix};

/// Error type for conflicts when merging fixes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClobberingError {
    DifferentReplacements {
        range: (usize, usize),
        existing_content: String,
        new_content: String,
    },
    SwallowedInsertion {
        insertion_pos: usize,
        deletion_range: (usize, usize),
    },
    Overlapping {
        existing: (usize, usize),
        new: (usize, usize),
    },
}

/// Corrector that merges multiple fixes and applies them to source code.
#[derive(Debug, Default)]
pub struct Corrector {
    edits: Vec<Edit>,
}
impl Corrector {
    /// Create a new empty corrector.
    pub fn new() -> Self {
        Self { edits: Vec::new() }
    }
    /// Merge a fix into this corrector.
    pub fn merge(&mut self, fix: &Fix) -> Result<(), ClobberingError> {
        for new_edit in &fix.edits {
            self.check_conflict(new_edit)?;
        }
        // No conflicts, add all edits
        self.edits.extend(fix.edits.iter().cloned());
        // Keep sorted for efficient application
        self.edits.sort_by_key(|e| (e.start, e.end));
        Ok(())
    }
    /// Check if a new edit conflicts with existing edits.
    fn check_conflict(&self, new_edit: &Edit) -> Result<(), ClobberingError> {
        for existing_edit in &self.edits {
            // 1. Same range with different content
            if existing_edit.start == new_edit.start && existing_edit.end == new_edit.end {
                if existing_edit.content != new_edit.content {
                    return Err(ClobberingError::DifferentReplacements {
                        range: (existing_edit.start, existing_edit.end),
                        existing_content: existing_edit.content.clone(),
                        new_content: new_edit.content.clone(),
                    });
                }
                continue;
            }
            // 2. Insertion swallowed by deletion
            // New edit is an insertion (start == end) inside an existing deletion
            if new_edit.start == new_edit.end && existing_edit.content.is_empty() && existing_edit.start < new_edit.start && new_edit.start < existing_edit.end
            {
                return Err(ClobberingError::SwallowedInsertion {
                    insertion_pos: new_edit.start,
                    deletion_range: (existing_edit.start, existing_edit.end),
                });
            }
            // Also check reverse: existing insertion inside new deletion
            if existing_edit.start == existing_edit.end
                && new_edit.content.is_empty()
                && new_edit.start < existing_edit.start
                && existing_edit.start < new_edit.end
            {
                return Err(ClobberingError::SwallowedInsertion {
                    insertion_pos: existing_edit.start,
                    deletion_range: (new_edit.start, new_edit.end),
                });
            }
            // 3. Overlapping ranges (not identical)
            // RuboCop uses `crossing_deletions: :accept` which merges overlapping deletions.
            // We instead reject overlaps and let the next iteration handle them.
            // This is simpler and produces the same final result, though may require
            // more iterations in rare cases.
            if ranges_overlap(existing_edit.start, existing_edit.end, new_edit.start, new_edit.end) {
                return Err(ClobberingError::Overlapping {
                    existing: (existing_edit.start, existing_edit.end),
                    new: (new_edit.start, new_edit.end),
                });
            }
        }

        Ok(())
    }
    /// Apply all merged edits to the source code.
    pub fn apply(&self, source: &[u8]) -> Vec<u8> {
        // If no edits, return original source
        if self.edits.is_empty() {
            return source.to_vec();
        }
        let mut result = Vec::with_capacity(source.len());
        let mut last_pos = 0;
        for edit in &self.edits {
            // Copy unchanged content before this edit
            if last_pos < edit.start {
                result.extend_from_slice(&source[last_pos..edit.start]);
            }
            // Apply the edit
            result.extend_from_slice(edit.content.as_bytes());
            last_pos = edit.end;
        }
        // Copy remaining content after last edit
        if last_pos < source.len() {
            result.extend_from_slice(&source[last_pos..]);
        }
        result
    }
    /// Returns the number of edits that will be applied.
    pub fn edit_count(&self) -> usize {
        self.edits.len()
    }
    /// Returns true if there are no edits to apply.
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }
}

/// Check if two ranges overlap.
fn ranges_overlap(start1: usize, end1: usize, start2: usize, end2: usize) -> bool {
    // Ranges overlap if one starts before the other ends
    // [start1, end1) and [start2, end2)
    start1 < end2 && start2 < end1
}

/// Check if a fix should be applied based on its applicability.
pub fn should_apply_fix(fix: &Fix, unsafe_fixes: bool) -> bool {
    match fix.applicability {
        Applicability::Safe => true,
        Applicability::Unsafe => unsafe_fixes,
        Applicability::DisplayOnly => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_edit(start: usize, end: usize, content: &str) -> Edit {
        Edit {
            start,
            end,
            content: content.to_string(),
        }
    }

    fn make_fix(edits: Vec<Edit>) -> Fix {
        Fix {
            applicability: Applicability::Safe,
            edits,
        }
    }

    #[test]
    fn test_merge_no_conflict() {
        let mut corrector = Corrector::new();

        let fix1 = make_fix(vec![make_edit(0, 5, "hello")]);
        let fix2 = make_fix(vec![make_edit(10, 15, "world")]);

        assert!(corrector.merge(&fix1).is_ok());
        assert!(corrector.merge(&fix2).is_ok());
        assert_eq!(corrector.edit_count(), 2);
    }

    #[test]
    fn test_merge_different_replacements() {
        let mut corrector = Corrector::new();

        let fix1 = make_fix(vec![make_edit(0, 5, "hello")]);
        let fix2 = make_fix(vec![make_edit(0, 5, "world")]); // Same range, different content

        assert!(corrector.merge(&fix1).is_ok());
        let result = corrector.merge(&fix2);

        assert!(matches!(result, Err(ClobberingError::DifferentReplacements { .. })));
        assert_eq!(corrector.edit_count(), 1); // Only first fix applied
    }

    #[test]
    fn test_merge_same_replacement() {
        let mut corrector = Corrector::new();

        let fix1 = make_fix(vec![make_edit(0, 5, "hello")]);
        let fix2 = make_fix(vec![make_edit(0, 5, "hello")]); // Same range, same content

        assert!(corrector.merge(&fix1).is_ok());
        assert!(corrector.merge(&fix2).is_ok()); // Should succeed (idempotent)
        assert_eq!(corrector.edit_count(), 2); // Both added (will produce same result)
    }

    #[test]
    fn test_merge_overlapping() {
        let mut corrector = Corrector::new();

        let fix1 = make_fix(vec![make_edit(0, 10, "hello")]);
        let fix2 = make_fix(vec![make_edit(5, 15, "world")]); // Overlaps with fix1

        assert!(corrector.merge(&fix1).is_ok());
        let result = corrector.merge(&fix2);

        assert!(matches!(result, Err(ClobberingError::Overlapping { .. })));
        assert_eq!(corrector.edit_count(), 1);
    }

    #[test]
    fn test_merge_swallowed_insertion() {
        let mut corrector = Corrector::new();

        // Deletion from 0 to 10
        let fix1 = make_fix(vec![make_edit(0, 10, "")]);
        // Insertion at position 5 (inside the deletion)
        let fix2 = make_fix(vec![make_edit(5, 5, "inserted")]);

        assert!(corrector.merge(&fix1).is_ok());
        let result = corrector.merge(&fix2);

        assert!(matches!(result, Err(ClobberingError::SwallowedInsertion { .. })));
    }

    #[test]
    fn test_apply_simple() {
        let mut corrector = Corrector::new();
        let source = b"hello world";

        let fix = make_fix(vec![make_edit(6, 11, "rust")]);
        corrector.merge(&fix).unwrap();

        let result = corrector.apply(source);
        assert_eq!(result, b"hello rust");
    }

    #[test]
    fn test_apply_multiple_edits() {
        let mut corrector = Corrector::new();
        let source = b"aaa bbb ccc";

        let fix1 = make_fix(vec![make_edit(0, 3, "AAA")]);
        let fix2 = make_fix(vec![make_edit(8, 11, "CCC")]);

        corrector.merge(&fix1).unwrap();
        corrector.merge(&fix2).unwrap();

        let result = corrector.apply(source);
        assert_eq!(result, b"AAA bbb CCC");
    }

    #[test]
    fn test_apply_deletion() {
        let mut corrector = Corrector::new();
        let source = b"hello   world"; // Extra spaces

        let fix = make_fix(vec![make_edit(5, 8, " ")]); // Replace 3 spaces with 1
        corrector.merge(&fix).unwrap();

        let result = corrector.apply(source);
        assert_eq!(result, b"hello world");
    }

    #[test]
    fn test_apply_insertion() {
        let mut corrector = Corrector::new();
        let source = b"helloworld";

        let fix = make_fix(vec![make_edit(5, 5, " ")]); // Insert space
        corrector.merge(&fix).unwrap();

        let result = corrector.apply(source);
        assert_eq!(result, b"hello world");
    }

    #[test]
    fn test_apply_empty() {
        let corrector = Corrector::new();
        let source = b"hello world";

        let result = corrector.apply(source);
        assert_eq!(result, source);
    }

    #[test]
    fn test_ranges_overlap() {
        // No overlap
        assert!(!ranges_overlap(0, 5, 5, 10));
        assert!(!ranges_overlap(5, 10, 0, 5));

        // Overlap
        assert!(ranges_overlap(0, 10, 5, 15));
        assert!(ranges_overlap(5, 15, 0, 10));

        // Contained
        assert!(ranges_overlap(0, 20, 5, 15));
        assert!(ranges_overlap(5, 15, 0, 20));

        // Adjacent (no overlap)
        assert!(!ranges_overlap(0, 5, 5, 10));
    }

    #[test]
    fn test_should_apply_fix() {
        let safe_fix = Fix::safe(vec![]);
        let unsafe_fix = Fix::r#unsafe(vec![]);
        let display_fix = Fix::display_only(vec![]);

        // Without unsafe_fixes
        assert!(should_apply_fix(&safe_fix, false));
        assert!(!should_apply_fix(&unsafe_fix, false));
        assert!(!should_apply_fix(&display_fix, false));

        // With unsafe_fixes
        assert!(should_apply_fix(&safe_fix, true));
        assert!(should_apply_fix(&unsafe_fix, true));
        assert!(!should_apply_fix(&display_fix, true));
    }
}
