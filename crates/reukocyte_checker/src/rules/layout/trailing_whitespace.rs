use crate::Checker;
use crate::diagnostic::Edit;
use crate::diagnostic::Fix;
use crate::locator::Line;
use crate::rule::Check;
use crate::rule::LayoutRule;
use crate::rule::Rule;
use crate::rule::RuleId;
use reukocyte_macros::check;

/// Rule identifier for Layout/TrailingWhitespace.
pub struct TrailingWhitespace;
impl Rule for TrailingWhitespace {
    const ID: RuleId = RuleId::Layout(LayoutRule::TrailingWhitespace);
}

/// Check for trailing whitespace in the source.
///
/// This rule doesn't need AST information - it operates on raw source bytes.
/// Directly pushes diagnostics to the Checker (Ruff-style).
#[check(Line)]
impl Check<Line<'_>> for TrailingWhitespace {
    fn check(_line: &Line, _checker: &mut Checker) {
        let text = _line.text;
        let mut count = 0;
        for b in text.iter().rev() {
            if *b == b' ' || *b == b'\t' {
                count += 1;
                continue;
            }
            break;
        }

        if count != 0 {
            let line_end = _line.end;
            let report_start = line_end - count;
            let report_end = line_end;
            let fix = Fix::safe(vec![Edit::deletion(report_start, report_end)]);
            _checker.report(
                TrailingWhitespace::ID,
                "xxxx".to_string(),
                crate::Severity::Convention,
                report_start,
                report_end,
                Some(fix),
            );
        }
    }
}
