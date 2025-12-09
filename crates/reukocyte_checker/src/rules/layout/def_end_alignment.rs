use crate::checker::Checker;
use crate::config::layout::def_end_alignment::EnforcedStyleAlignWith;
use crate::diagnostic::Edit;
use crate::diagnostic::Fix;
use crate::rule::Check;
use crate::rule::LayoutRule;
use crate::rule::Rule;
use crate::rule::RuleId;
use crate::utility::access_modifier::is_access_modifier;
use reukocyte_macros::check;
use ruby_prism::*;

/// Layout/DefEndAlignment rule.
pub struct DefEndAlignment;
impl Rule for DefEndAlignment {
    const ID: RuleId = RuleId::Layout(LayoutRule::DefEndAlignment);
}
#[check(DefNode)]
impl Check<DefNode<'_>> for DefEndAlignment {
    fn check(node: &DefNode, checker: &mut Checker) {
        let call_node_id = checker
            .semantic()
            .ancestor_with_id(1)
            .and_then(|(id, node)| if node.as_call_node().is_some() { Some(id) } else { None });
        let def = node.def_keyword_loc().start_offset();
        let start_of_line = checker.line_index().indentation(def);
        // FIXME: is_access_modifierをここで呼んでしまうと、DefNodeがマクロスコープにないので正しく判定できない

        if call_node_id.is_some_and(|id| is_access_modifier(&id, checker)) {
            check_end_kw_alignment(node, def, start_of_line, checker);
        } else {
            check_end_kw_alignment(node, def, def, checker);
        }
    }
}

fn check_end_kw_alignment(node: &DefNode, def: usize, start_of_line: usize, checker: &mut Checker) {
    if checker.is_ignored_node(node.location().start_offset(), node.location().end_offset()) {
        return;
    }
    if let Some(end_keyword_loc) = node.end_keyword_loc() {
        let line_index = checker.line_index();
        let column_delta = match checker.config().layout.def_end_alignment.enforced_style_align_with {
            EnforcedStyleAlignWith::StartOfLine => {
                let are_same_line = line_index.are_on_same_line(start_of_line, end_keyword_loc.start_offset());
                if are_same_line {
                    return;
                }
                let column_delta = line_index.column_offset_between(start_of_line, end_keyword_loc.start_offset());
                if column_delta == 0 {
                    return;
                }
                column_delta
            }
            EnforcedStyleAlignWith::Def => {
                let are_same_line = line_index.are_on_same_line(def, end_keyword_loc.start_offset());
                if are_same_line {
                    return;
                }
                let column_delta = line_index.column_offset_between(def, end_keyword_loc.start_offset());
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
            DefEndAlignment::ID,
            format!("`end` keyword should be aligned with its opening keyword."),
            crate::diagnostic::Severity::Convention,
            end_keyword_loc.start_offset(),
            end_keyword_loc.end_offset(),
            Some(fix),
        );
    }
}
