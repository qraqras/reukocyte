pub mod debugger;

/// Lint cop configurations.
#[derive(Debug, Clone, Default)]
pub struct LintConfig {
    pub debugger: debugger::Debugger,
}
