use crate::Checker;
use crate::rule::Check;
use crate::rule::LayoutRule;
use crate::rule::Line;
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
        //
    }
}
