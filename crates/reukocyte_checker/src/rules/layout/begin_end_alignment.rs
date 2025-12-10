use crate::checker::Checker;
use crate::config::layout::begin_end_alignment::EnforcedStyleAlignWith;
use crate::diagnostic::Edit;
use crate::diagnostic::Fix;
use crate::rule::Check;
use crate::rule::LayoutRule;
use crate::rule::Rule;
use crate::rule::RuleId;
use reukocyte_macros::check;
use ruby_prism::*;

/// Get the config for this rule
#[inline]
fn config<'a>(checker: &'a Checker<'_>) -> &'a crate::config::layout::begin_end_alignment::BeginEndAlignment {
    &checker.config().layout.begin_end_alignment
}

/// Layout/BeginEndAlignment rule.
pub struct BeginEndAlignment;
impl Rule for BeginEndAlignment {
    const ID: RuleId = RuleId::Layout(LayoutRule::BeginEndAlignment);
}
#[check(BeginNode)]
impl Check<BeginNode<'_>> for BeginEndAlignment {
    fn check(node: &BeginNode, checker: &mut Checker) {
        check_end_kw_alignment(node, checker);
    }
}

fn check_end_kw_alignment(node: &BeginNode, checker: &mut Checker) {
    if checker.is_ignored_node(node.location().start_offset(), node.location().end_offset()) {
        return;
    }
    let cfg = config(checker);
    if let Some(end_keyword_loc) = node.end_keyword_loc() {
        let column_delta = match cfg.enforced_style_align_with {
            EnforcedStyleAlignWith::StartOfLine => {
                let start_of_line = checker.line_index().indentation(node.location().start_offset());
                let are_same_line = checker.line_index().are_on_same_line(start_of_line, end_keyword_loc.start_offset());
                if are_same_line {
                    return;
                }
                let column_delta = checker.line_index().column_offset_between(start_of_line, end_keyword_loc.start_offset());
                if column_delta == 0 {
                    return;
                }
                column_delta
            }
            EnforcedStyleAlignWith::Begin => {
                let Some(begin_keyword_loc) = node.begin_keyword_loc() else {
                    return;
                };
                let are_same_line = checker
                    .line_index()
                    .are_on_same_line(begin_keyword_loc.start_offset(), end_keyword_loc.start_offset());
                if are_same_line {
                    return;
                }
                let column_delta = checker
                    .line_index()
                    .column_offset_between(begin_keyword_loc.start_offset(), end_keyword_loc.start_offset());
                if column_delta == 0 {
                    return;
                }
                column_delta
            }
        };

        let fix = if column_delta > 0 {
            let end = end_keyword_loc.start_offset();
            let start = end as i32 - column_delta;
            Fix::safe(vec![Edit::deletion(start as usize, end)])
        } else {
            Fix::safe(vec![Edit::insertion(end_keyword_loc.start_offset(), " ".repeat(column_delta.abs() as usize))])
        };

        checker.report(
            BeginEndAlignment::ID,
            format!("`end` keyword should be aligned with its opening keyword."),
            cfg.severity,
            end_keyword_loc.start_offset(),
            end_keyword_loc.end_offset(),
            Some(fix),
        );
    }
}
