use crate::semantic::Indexer;
mod checker;
mod config;
mod conflict;
mod corrector;
pub mod custom_nodes;
mod diagnostic;
mod fix;
mod locator;
mod rule;
mod semantic;
pub mod utility;

pub mod rules;

pub use checker::Checker;
pub use config::{AllCopsConfig, Config, InheritFrom, LayoutConfig, LoadError, RubocopYaml, load_rubocop_yaml, parse_rubocop_yaml};

pub use conflict::ConflictRegistry;
pub use corrector::{ClobberingError, Corrector};
pub use diagnostic::{Applicability, Diagnostic, Edit, Fix, Severity};
pub use fix::{InfiniteCorrectionLoop, apply_fixes, apply_fixes_filtered, apply_fixes_with_loop_detection, apply_fixes_with_remaining};
pub use locator::LineIndex;
pub use rule::{Category, Check, LayoutRule, LintRule, Rule, RuleId};
use std::path::Path;

/// Check a Ruby source file for violations with default configuration.
///
/// This is the main entry point that:
/// 1. Parses the source once
/// 2. Traverses the AST once for all node-based rules (Lint)
/// 3. Runs line-based rules (Layout) - can use info from AST phase
pub fn check(source: &[u8]) -> Vec<Diagnostic> {
    check_with_config(source, &Config::default())
}

/// Check a Ruby source file for violations with custom configuration.
pub fn check_with_config(source: &[u8], config: &Config) -> Vec<Diagnostic> {
    check_with_config_and_path(source, config, None)
}

/// Check a Ruby source file for violations with custom configuration and file path.
///
/// The file path is used for cop-specific Exclude pattern matching.
pub fn check_with_config_and_path(source: &[u8], config: &Config, file_path: Option<&Path>) -> Vec<Diagnostic> {
    // Profiling removed — no `Instant` usage

    let mut checker = Checker::new(source, config, file_path);

    // Decide whether to parse based on which rule categories are enabled.
    // If any AST/token based rules are enabled, we need to parse; otherwise skip.
    let needs_parse = true;
    let parse_result = if needs_parse { Some(ruby_prism::parse(source)) } else { None };

    // Phase 1: Run AST-based rules (single traversal with semantic model building)
    if true {
        if let Some(ref parse_result) = parse_result {
            let node = parse_result.node();
            checker.semantic = Indexer::new().index(&node);
            let node2 = parse_result.node();
            checker.visit_nodes(node2);
        }
    }

    // Phase 2: Run line-based rules (after AST, can use collected info)
    if true {
        checker.visit_lines();
        // No profiling: directly run line-based checks
    }

    // Some layout checks need file-level analysis; keep them executed explicitly
    if true {
        // checker.run_file_rules(); // No file rules currently
    }

    let res = checker.into_diagnostics();
    // No profiling timing output
    // Print aggregated per-rule timing if profiling enabled
    // Profiling output disabled
    // Temporarily disabled due to SemanticModel removal
    //             eprintln!(
    //                 "[rule_subphase] layout::indentation_consistency::IndentationConsistency,total_us,count,collect_us,alignment_us,offsets_us,batch_us,iter_us,fix_creation_us,report_us,conflict_us"
    //             );
    //             eprintln!(
    //                 "[rule_subphase] layout::indentation_consistency::IndentationConsistency,{},{},{},{},{},{},{},{},{},{}",
    //                 r.total_us, r.count, r.collect_us, r.align_us, r.offsets_us, r.batch_us, r.iter_us, r.fix_creation_us, r.report_us, r.conflict_us
    //             );
    //         }
    //         None => {
    //             eprintln!("[dbg] indentation_consistency sub-registry not initialized");
    //         }
    //     }
    // }
    res
}
