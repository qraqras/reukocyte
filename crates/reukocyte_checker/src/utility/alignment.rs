use crate::checker::Checker;
use crate::diagnostic::Edit;
use crate::diagnostic::Fix;
use crate::diagnostic::Severity;
use crate::rule::RuleId;
use ruby_prism::Location;

/// Check alignment of given locations against a base column.
pub fn check_alignment(locs: Vec<Location>, base_column: Option<usize>, rule_id: RuleId, checker: &mut Checker) {
    if locs.is_empty() {
        return;
    }

    let line_index = checker.line_index();
    let base_column = match base_column {
        Some(col) => col,
        None => line_index.column_number(locs.first().unwrap().start_offset()),
    };

    let mut reports = Vec::new();
    let mut prev_line = 0;
    for curr_loc in locs {
        let curr_line = line_index.line_number(curr_loc.start_offset());
        if prev_line < curr_line && line_index.is_first_on_line(curr_loc.start_offset()) {
            let column_delta = base_column as isize - line_index.column_number(curr_loc.start_offset()) as isize;
            if column_delta != 0 {
                let line_start = line_index.line_start_offset(curr_loc.start_offset());
                let fix = if column_delta > 0 {
                    Fix::safe(Vec::from([Edit::insertion(line_start, " ".repeat(column_delta as usize))]))
                } else {
                    let remove_count = (-column_delta) as usize;
                    let end = line_start + remove_count.min(curr_loc.start_offset() - line_start);
                    Fix::safe(Vec::from([Edit::deletion(line_start, end)]))
                };
                reports.push((curr_loc.start_offset(), curr_loc.end_offset(), fix));
            }
        }
        prev_line = curr_line;
    }

    for (start, end, fix) in reports {
        // TODO: message
        checker.report(rule_id, "".to_string(), Severity::Convention, start, end, Some(fix));
    }
}
