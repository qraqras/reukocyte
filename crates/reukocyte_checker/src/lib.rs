mod checker;
mod config;
mod conflict;
mod corrector;
pub mod custom_nodes;
mod diagnostic;
mod fix;
mod locator;
mod rule;
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
pub fn check_with_config_and_path(source: &[u8], config: &Config, file_path: Option<&str>) -> Vec<Diagnostic> {
    use std::env;
    use std::time::Instant;

    let profile_phases = env::var("RUEKO_PROFILE_PHASES").is_ok();
    let profile_t0 = if profile_phases { Some(Instant::now()) } else { None };

    let mut checker = if let Some(path) = file_path {
        Checker::with_file_path(source, config, path)
    } else {
        Checker::new(source, config)
    };

    // Decide whether to parse based on which rule categories are enabled.
    // If any AST/token based rules are enabled, we need to parse; otherwise skip.
    let needs_parse = true;
    let parse_result = if needs_parse {
        let parse_start = if profile_phases { Some(Instant::now()) } else { None };
        let res = ruby_prism::parse(source);
        if profile_phases {
            if let Some(dur) = parse_start.map(|s| s.elapsed()) {
                eprintln!("[phase] parse: {} ms", dur.as_millis());
            }
        }
        Some(res)
    } else {
        None
    };

    // Phase 1: Run AST-based rules (single traversal with semantic model building)
    if true {
        if let Some(ref parse_result) = parse_result {
            let visit_nodes_start = if profile_phases { Some(Instant::now()) } else { None };
            checker.visit_nodes(&parse_result.node());
            if profile_phases {
                if let Some(dur) = visit_nodes_start.map(|s| s.elapsed()) {
                    eprintln!("[phase] visit_nodes: {} ms", dur.as_millis());
                }
            }
        }
    }

    // Phase 2: Run line-based rules (after AST, can use collected info)
    if true {
        let visit_lines_start = if profile_phases { Some(Instant::now()) } else { None };
        checker.visit_lines();
        if profile_phases {
            if let Some(dur) = visit_lines_start.map(|s| s.elapsed()) {
                eprintln!("[phase] visit_lines: {} ms", dur.as_millis());
            }
        }
    }

    // Some layout checks need file-level analysis; keep them executed explicitly
    let trailing_start = if profile_phases { Some(Instant::now()) } else { None };
    if true {
        // checker.run_file_rules(); // No file rules currently
    }
    if profile_phases {
        if let Some(dur) = trailing_start.map(|s| s.elapsed()) {
            eprintln!("[phase] trailing_empty_lines: {} ms", dur.as_millis());
        }
    }

    let into_diagnostics_start = if profile_phases { Some(Instant::now()) } else { None };
    let res = checker.into_diagnostics();
    if profile_phases {
        if let Some(dur) = into_diagnostics_start.map(|s| s.elapsed()) {
            eprintln!("[phase] into_diagnostics: {} ms", dur.as_millis());
        }
        if let Some(start) = profile_t0 {
            eprintln!("[phase] total: {} ms", start.elapsed().as_millis());
        }
    }
    // Print aggregated per-rule timing if profiling enabled
    if std::env::var("RUEKO_PROFILE_RULES").is_ok() {
        // The registry is generated in the rule_registry macros and placed in the checker module
        if let Some(reg) = crate::checker::__REUKO_PROFILE_RULES_REGISTRY.get() {
            let map = reg.lock().unwrap();
            eprintln!("[rule_agg] rule_name,total_us,count");
            for (k, v) in map.iter() {
                eprintln!("[rule_agg] {},{},{}", k, v.0, v.1);
            }
        }
    }
    // Temporarily disabled due to SemanticModel removal
    // if std::env::var("RUEKO_PROFILE_RULE_SUBPHASES").is_ok() {
    //     eprintln!("[dbg] subphase env set");
    //     match crate::rules::layout::indentation_consistency::__REUKO_INDENTCONS_SUB_REGISTRY.get() {
    //         Some(reg) => {
    //             let r = reg.lock().unwrap();
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
