//! PreCop Core
//!
//! Core functionality for PreCop including:
//! - Configuration parsing (.rubocop.yml)
//! - Cop trait and base implementations
//! - Offense reporting
//! - Runner for executing cops

pub mod config;
pub mod cop;
pub mod offense;
pub mod runner;

pub use config::Config;
pub use cop::{AstCop, CheckContext, Cop, OffenseCollector};
pub use offense::Offense;
