pub mod debugger;

// Re-export config types
pub use debugger::DebuggerConfig;

/// Lint cop configurations.
#[derive(Debug, Clone, Default)]
pub struct LintConfig {
    pub debugger: debugger::DebuggerConfig,
}
