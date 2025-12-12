pub mod debugger;

/// Lint rule configurations.
#[derive(Debug, Clone, Default)]
pub struct LintConfig {
    pub debugger: debugger::Debugger,
}
