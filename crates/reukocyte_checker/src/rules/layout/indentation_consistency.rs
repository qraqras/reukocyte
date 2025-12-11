use crate::checker::Checker;
use crate::config::layout::indentation_consistency::EnforcedStyle;
use crate::rule::Check;
use crate::rule::LayoutRule;
use crate::rule::Rule;
use crate::rule::RuleId;
use crate::utility::access_modifier::is_bare_access_modifier;
use crate::utility::alignment::check_alignment;
use reukocyte_macros::check;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// Subphase registry for IndentationConsistency profiling.
/// Fields: total_us, count, collect_us, alignment_us,
/// offsets_us, batch_us, iter_us, fix_creation_us, report_us, conflict_us
#[derive(Default, Debug)]
pub(crate) struct IndentConsSubRegistry {
    pub total_us: u128,
    pub count: u64,
    pub collect_us: u128,
    pub align_us: u128,
    pub offsets_us: u128,
    pub batch_us: u128,
    pub iter_us: u128,
    pub fix_creation_us: u128,
    pub report_us: u128,
    pub conflict_us: u128,
}

pub(crate) static __REUKO_INDENTCONS_SUB_REGISTRY: OnceLock<Mutex<IndentConsSubRegistry>> = OnceLock::new();
use ruby_prism::*;

/// Get the config for this rule
#[inline]
fn config<'a>(checker: &'a Checker<'_>) -> &'a crate::config::layout::indentation_consistency::IndentationConsistency {
    &checker.config().layout.indentation_consistency
}

/// Layout/IndentationConsistency rule.
pub struct IndentationConsistency;
impl Rule for IndentationConsistency {
    const ID: RuleId = RuleId::Layout(LayoutRule::IndentationConsistency);
}
#[check(StatementsNode)]
impl Check<StatementsNode<'_>> for IndentationConsistency {
    fn check(node: &StatementsNode, checker: &mut Checker) {
        // Optional per-rule subphase profiling controlled by env var RUEKO_PROFILE_RULE_SUBPHASES
        let profile_sub = std::env::var("RUEKO_PROFILE_RULE_SUBPHASES").is_ok();
        if profile_sub {
            __REUKO_INDENTCONS_SUB_REGISTRY.get_or_init(|| Mutex::new(IndentConsSubRegistry::default()));
        }
        let sub_t0 = if profile_sub { Some(Instant::now()) } else { None };

        match config(checker).enforced_style {
            EnforcedStyle::Normal => check_normal_style(node, checker),
            EnforcedStyle::IndentedInternalMethods => check_indented_internal_methods_style(node, checker),
        }

        if profile_sub {
            if let Some(total) = sub_t0.map(|s| s.elapsed().as_micros() as u128) {
                let map = __REUKO_INDENTCONS_SUB_REGISTRY.get_or_init(|| Mutex::new(IndentConsSubRegistry::default()));
                let mut lock = map.lock().unwrap();
                lock.total_us += total;
                lock.count += 1;
                // The collect and alignment totals are maintained in their respective functions
                // by updating the registry entries lock.2 and lock.3 directly.
            }
        }
    }
}

/// Check indentation consistency in normal style.
fn check_normal_style(node: &StatementsNode, checker: &mut Checker) {
    // Quick pass: count the number of relevant children; if <=1 skip alignment checks.
    let mut target_count = 0usize;
    for child in node.body().iter() {
        if let Some(node_id) = checker.semantic().node_id_for(&child) {
            if !is_bare_access_modifier(&node_id, checker) {
                target_count += 1;
                if target_count > 1 {
                    break;
                }
            }
        }
    }
    if target_count <= 1 {
        return;
    }

    // Optionally profile target collection and alignment time
    let profile_sub = std::env::var("RUEKO_PROFILE_RULE_SUBPHASES").is_ok();
    let mut collect_us = 0u128;
    let mut align_us = 0u128;
    let mut align_detail = crate::utility::alignment::AlignmentProfile::default();
    let t_collect = if profile_sub { Some(Instant::now()) } else { None };

    let targets = node
        .body()
        .iter()
        .filter_map(|child| {
            let node_id = checker.semantic().node_id_for(&child)?;
            if !is_bare_access_modifier(&node_id, checker) {
                Some(child.location())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if profile_sub {
        if let Some(s) = t_collect.map(|s| s.elapsed().as_micros() as u128) { collect_us += s }
    }
    let t_align = if profile_sub { Some(Instant::now()) } else { None };
    check_alignment(
        targets,
        base_column_for_normal_style(node, checker),
        IndentationConsistency::ID,
        checker,
        if profile_sub { Some(&mut align_detail) } else { None },
    );
    if profile_sub {
        if let Some(s) = t_align.map(|s| s.elapsed().as_micros() as u128) { align_us += s }
        // Update module counters
        if let Some(m) = __REUKO_INDENTCONS_SUB_REGISTRY.get() {
            let mut lock = m.lock().unwrap();
            lock.collect_us += collect_us;
            lock.align_us += align_us;
            // also update detailed alignment breakdown
            lock.offsets_us += align_detail.offsets_us;
            lock.batch_us += align_detail.batch_us;
            lock.iter_us += align_detail.iter_us;
            lock.fix_creation_us += align_detail.fix_creation_us;
            lock.report_us += align_detail.report_us;
            lock.conflict_us += align_detail.conflict_check_us;
        }
    }
}

/// Check indentation consistency in indented internal methods style.
fn check_indented_internal_methods_style(node: &StatementsNode, checker: &mut Checker) {
    let mut children_to_check = Vec::new();
    let profile_sub = std::env::var("RUEKO_PROFILE_RULE_SUBPHASES").is_ok();
    let mut collect_us = 0u128;
    let mut align_us = 0u128;
    let mut align_detail = crate::utility::alignment::AlignmentProfile::default();
    let t_collect = if profile_sub { Some(Instant::now()) } else { None };
    for statement in node.body().iter() {
        let Some(node_id) = checker.semantic().node_id_for(&statement) else {
            continue;
        };
        if is_bare_access_modifier(&node_id, checker) {
            children_to_check.push(Vec::new());
        } else {
            if let Some(last_group) = children_to_check.last_mut() {
                last_group.push(statement.location());
            }
        }
    }
    for group in children_to_check {
        let t_align = if profile_sub { Some(Instant::now()) } else { None };
        check_alignment(group, None, IndentationConsistency::ID, checker, if profile_sub { Some(&mut align_detail) } else { None });
        if profile_sub {
            if let Some(s) = t_align.map(|s| s.elapsed().as_micros() as u128) { align_us += s }
        }
    }
    if profile_sub {
        if let Some(s) = t_collect.map(|s| s.elapsed().as_micros() as u128) { collect_us += s }
        if let Some(m) = __REUKO_INDENTCONS_SUB_REGISTRY.get() {
            let mut lock = m.lock().unwrap();
            lock.collect_us += collect_us;
            lock.align_us += align_us;
            lock.offsets_us += align_detail.offsets_us;
            lock.batch_us += align_detail.batch_us;
            lock.iter_us += align_detail.iter_us;
            lock.fix_creation_us += align_detail.fix_creation_us;
            lock.report_us += align_detail.report_us;
            lock.conflict_us += align_detail.conflict_check_us;
        }
    }
}

/// Determine the base column for normal style indentation checking.
fn base_column_for_normal_style(node: &StatementsNode, checker: &mut Checker) -> Option<usize> {
    let first_child = node.body().iter().next();
    if let Some(first_child) = first_child
        && let Some(node_id) = checker.semantic().node_id_for(&first_child)
        && is_bare_access_modifier(&node_id, checker)
    {
        let access_modifier_indent = checker.line_index().column_number(first_child.location().start_offset());
        // If the StatementsNode is inside a module/class, ensure access modifier is more indented
        if let Some(parent) = checker.semantic().parent() {
            let module_indent = checker.line_index().column_number(parent.location().start_offset());
            if module_indent < access_modifier_indent {
                return Some(access_modifier_indent);
            }
        } else {
            return Some(access_modifier_indent);
        }
    }
    None
}
