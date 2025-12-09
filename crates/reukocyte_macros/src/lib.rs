//! Procedural macros for reukocyte rule registration.
//!
//! This crate provides the `#[check(NodeType)]` attribute macro
//! for registering rule implementations with the checker.

use proc_macro::TokenStream;

/// Marker attribute for registering a rule's Check implementation.
///
/// This attribute marks an `impl Check<NodeType<'_>> for RuleName` block
/// for automatic discovery by the build script. The attribute itself
/// does nothing at runtime - it simply passes through the impl block unchanged.
///
/// # Example
///
/// ```ignore
/// #[check(CallNode)]
/// impl Check<CallNode<'_>> for Debugger {
///     fn check(node: &CallNode, checker: &mut Checker) {
///         // ...
///     }
/// }
/// ```
///
/// The build script scans for `#[check(NodeType)]` patterns and generates
/// the appropriate dispatch code.
#[proc_macro_attribute]
pub fn check(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Simply return the item unchanged - this is just a marker
    item
}
