pub mod layout;

pub use layout::*;

/// The main configuration struct.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub layout: LayoutConfig,
}
