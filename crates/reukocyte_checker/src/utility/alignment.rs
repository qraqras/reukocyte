use crate::checker::Checker;
use crate::diagnostic::Edit;
use crate::diagnostic::Fix;
use crate::diagnostic::Severity;
use crate::rule::RuleId;
use ruby_prism::Location;
use std::time::Instant;

/// Check alignment of given locations against a base column.
/// Optional profiling information for check_alignment subphases.
#[derive(Default)]
pub struct AlignmentProfile {
    pub offsets_us: u128,
    pub batch_us: u128,
    pub iter_us: u128,
    pub fix_creation_us: u128,
    pub report_us: u128,
    pub conflict_check_us: u128,
}

/// Check alignment of given locations against a base column.
/// If `profile` is Some, per-subphase timings will be written into it.
pub fn check_alignment(
    locs: Vec<Location>,
    base_column: Option<usize>,
    rule_id: RuleId,
    checker: &mut Checker,
    mut profile: Option<&mut AlignmentProfile>,
) {
    if locs.is_empty() {
        return;
    }
    // If there is only one target, there can be no alignment inconsistency.
    if locs.len() <= 1 {
        return;
    }

    let base_column = match base_column {
        Some(col) => col,
        None => checker.line_index().column_number(locs.first().unwrap().start_offset()),
    };

    // Collect offsets for batch processing to avoid repeated binary searches.
    let mut offsets: Vec<(usize, usize)> = Vec::with_capacity(locs.len());
    let t_offsets = if profile.is_some() { Some(Instant::now()) } else { None };
    for loc in &locs {
        offsets.push((loc.start_offset(), loc.end_offset()));
    }
        if let Some(p) = profile.as_mut() {
            if let Some(s) = t_offsets.map(|s| s.elapsed().as_micros() as u128) { p.offsets_us += s }
        }
    let t_batch = if profile.is_some() { Some(Instant::now()) } else { None };
    let resolved = checker.line_index().batch_line_info(&offsets);
    if let Some(p) = profile.as_mut() {
        if let Some(s) = t_batch.map(|s| s.elapsed().as_micros() as u128) { p.batch_us += s }
    }

    enum PendingEdit {
        Insert(usize),
        Delete(usize),
    }
    let mut reports: Vec<(usize, usize, PendingEdit)> = Vec::new();
    let mut prev_line: usize = 0;
    for (i, curr_loc) in locs.into_iter().enumerate() {
        let t_iter = if profile.is_some() { Some(Instant::now()) } else { None };
        let (line_start, _line_end, col_start, _col_end, indent) = resolved[i];
        let curr_line = line_start; // line_start returned is 1-indexed (line number)
        if prev_line < curr_line && col_start - 1 == indent {
            let column_delta = base_column as isize - col_start as isize;
            if column_delta != 0 {
                // Removed unused line_start_offset variable
                if column_delta > 0 {
                    reports.push((curr_loc.start_offset(), curr_loc.end_offset(), PendingEdit::Insert(column_delta as usize)));
                } else {
                    let remove_count = (-column_delta) as usize;
                    reports.push((curr_loc.start_offset(), curr_loc.end_offset(), PendingEdit::Delete(remove_count)));
                }
            }
        }
        if let Some(p) = profile.as_mut() {
            if let Some(s) = t_iter.map(|s| s.elapsed().as_micros() as u128) { p.iter_us += s }
        }
        prev_line = curr_line;
    }

    // Avoid producing overlapping fixes to reduce conflicts during fix application.
    // RuboCop behavior: if an offense is within a region that's already going to be
    // realigned by an earlier correction, register the offense without an autocorrect
    // to avoid two rewrites in the same area.
    let mut last_applied_end: Option<usize> = None;
    for (start, end, pe) in reports {
        let t_conflict = if profile.is_some() { Some(Instant::now()) } else { None };
        // If this reported fix starts before the previously applied fix end, it's
        // overlapping. In that case, register without autocorrect (fix=None).
        if let Some(prev_end) = last_applied_end {
            if start < prev_end {
                checker.report(rule_id, "".to_string(), Severity::Convention, start, end, None);
                if let Some(p) = profile.as_mut() {
                    if let Some(s) = t_conflict.map(|s| s.elapsed().as_micros() as u128) { p.conflict_check_us += s }
                }
                continue;
            }
        }
        // Build the fix lazily to avoid allocations when skipped due to overlap.
        let t_fix = if profile.is_some() { Some(Instant::now()) } else { None };
        let fix = match pe {
            PendingEdit::Insert(n) => {
                let line_start_offset = checker.line_index().line_start_offset(start);
                Fix::safe(Vec::from([Edit::insertion(line_start_offset, " ".repeat(n))]))
            }
            PendingEdit::Delete(n) => {
                let line_start_offset = checker.line_index().line_start_offset(start);
                let endpos = line_start_offset + n.min(start - line_start_offset);
                Fix::safe(Vec::from([Edit::deletion(line_start_offset, endpos)]))
            }
        };
        if let Some(p) = profile.as_mut() {
            if let Some(s) = t_fix.map(|s| s.elapsed().as_micros() as u128) { p.fix_creation_us += s }
        }
        let t_report = if profile.is_some() { Some(Instant::now()) } else { None };
        checker.report(rule_id, "".to_string(), Severity::Convention, start, end, Some(fix));
        if let Some(p) = profile.as_mut() {
            if let Some(s) = t_report.map(|s| s.elapsed().as_micros() as u128) { p.report_us += s }
        }
        last_applied_end = Some(end);
    }
}
