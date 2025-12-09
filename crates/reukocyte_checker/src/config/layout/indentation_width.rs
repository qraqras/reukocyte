/// Configuration for Layout/IndentationWidth.
#[derive(Debug, Clone)]
pub struct IndentationWidthConfig {
    pub width: i32,
    pub allowed_patterns: Vec<i32>,
}
impl Default for IndentationWidthConfig {
    fn default() -> Self {
        Self {
            width: 2,
            allowed_patterns: Vec::new(),
        }
    }
}
