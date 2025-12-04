use ruby_prism::Node;

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

/// Returns the first part of a method call chain.
///
/// For example, given `foo.bar.baz`, this returns `foo`.
/// For `foo.bar { block }.baz.qux`, this also returns `foo`.
///
/// In Prism's AST, blocks are attached to CallNode as a `block` field,
/// not as a separate node in the receiver chain. So we only need to
/// traverse the `receiver` of CallNodes.
///
/// This is equivalent to RuboCop's `first_part_of_call_chain` in `lib/rubocop/cop/util.rb`.
pub fn first_part_of_call_chain(node: Node) -> Option<Node> {
    let mut current = Some(node);
    while let Some(node) = &current {
        if let Some(call_node) = node.as_call_node() {
            current = call_node.receiver();
            continue;
        }
        break;
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_blank_space() {
        assert!(is_blank(' '));
    }

    #[test]
    fn test_is_blank_tab() {
        assert!(is_blank('\t'));
    }

    #[test]
    fn test_is_blank_fullwidth_space() {
        assert!(is_blank('\u{3000}'));
    }

    #[test]
    fn test_is_blank_cr_not_blank() {
        assert!(!is_blank('\r'));
    }

    #[test]
    fn test_is_blank_newline_not_blank() {
        assert!(!is_blank('\n'));
    }

    #[test]
    fn test_is_blank_regular_char_not_blank() {
        assert!(!is_blank('a'));
        assert!(!is_blank('0'));
    }
}
