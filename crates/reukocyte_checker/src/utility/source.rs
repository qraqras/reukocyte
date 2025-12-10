/// Check if a character is a blank character (RuboCop's `[[:blank:]]`).
///
/// This matches: space, tab, and fullwidth space (U+3000).
/// Note: CR (`\r`) is NOT considered blank in RuboCop.
///
/// # Examples
///
/// ```
/// use reukocyte_checker::utils::is_blank;
///
/// assert!(is_blank(' '));
/// assert!(is_blank('\t'));
/// assert!(is_blank('\u{3000}')); // fullwidth space
/// assert!(!is_blank('\r'));
/// assert!(!is_blank('a'));
/// ```
#[inline]
pub fn is_blank(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\u{3000}')
}
