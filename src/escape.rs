use crate::lexer::{Lexer, SyntaxKind};

pub fn escape(source: &str) -> String {
    let lexer = Lexer::new(source);
    let mut result = String::with_capacity(source.len());

    for lexeme in lexer {
        if let SyntaxKind::ParenthesisOpen
        | SyntaxKind::ParenthesisClose
        | SyntaxKind::SquareBracketOpen
        | SyntaxKind::SquareBracketClose
        | SyntaxKind::EscapeCharacter
        | SyntaxKind::EscapeWhitespace = lexeme.kind
        {
            result.push('\\');
        }

        result.push_str(lexeme.text);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::escape;

    #[test]
    fn lowercase_alphabetic() {
        assert_eq!(escape("abcdef"), "abcdef");
    }

    #[test]
    fn uppercase_alphabetic() {
        assert_eq!(escape("ABCDEF"), "ABCDEF");
    }

    #[test]
    fn mixed_case_alphabetic() {
        assert_eq!(escape("aBcDeF"), "aBcDeF");
    }

    #[test]
    fn numeric() {
        assert_eq!(escape("123456"), "123456");
    }

    #[test]
    fn lowercase_alphanumeric() {
        assert_eq!(escape("abc123"), "abc123");
    }

    #[test]
    fn uppercase_alphanumeric() {
        assert_eq!(escape("ABC123"), "ABC123");
    }

    #[test]
    fn mixed_case_alphanumeric() {
        assert_eq!(escape("AbCd1234"), "AbCd1234");
    }

    #[test]
    fn special_characters() {
        assert_eq!(escape("!@#$%^"), "!@#$%^");
    }

    #[test]
    fn mixed_characters() {
        assert_eq!(escape("ABCdef123!@#"), "ABCdef123!@#");
    }

    #[test]
    fn escaped_characters() {
        assert_eq!(escape("\\(\\)\\[\\]"), "\\\\(\\\\)\\\\[\\\\]");
    }

    #[test]
    fn escaped_whitespace() {
        assert_eq!(escape("\\ \n\t"), "\\\\ \n\t");
    }

    #[test]
    fn empty_token() {
        assert_eq!(escape("[fg:red]()"), "\\[fg:red\\]\\(\\)");
    }

    #[test]
    fn token_with_foreground() {
        assert_eq!(escape("[fg:red](inner)"), "\\[fg:red\\]\\(inner\\)");
    }

    #[test]
    fn token_with_background() {
        assert_eq!(escape("[bg:blue](inner)"), "\\[bg:blue\\]\\(inner\\)");
    }

    #[test]
    fn token_with_single_decoration() {
        assert_eq!(escape("[deco:dim](inner)"), "\\[deco:dim\\]\\(inner\\)");
    }

    #[test]
    fn token_with_multiple_decorations() {
        assert_eq!(
            escape("[deco:dim,italic](inner)"),
            "\\[deco:dim,italic\\]\\(inner\\)"
        );
    }

    #[test]
    fn token_with_multiple_styles() {
        assert_eq!(
            escape("[deco:dim,italic;fg:red;bg:blue](inner)"),
            "\\[deco:dim,italic;fg:red;bg:blue\\]\\(inner\\)"
        );
    }

    #[test]
    fn token_with_leading_content() {
        assert_eq!(
            escape("leading [fg:red](content)"),
            "leading \\[fg:red\\]\\(content\\)"
        );
    }

    #[test]
    fn token_with_trailing_content() {
        assert_eq!(
            escape("[fg:red](content) trailing"),
            "\\[fg:red\\]\\(content\\) trailing"
        );
    }

    #[test]
    fn token_with_leading_and_trailing_content() {
        assert_eq!(
            escape("leading [fg:red](content) trailing"),
            "leading \\[fg:red\\]\\(content\\) trailing"
        );
    }

    #[test]
    fn nested_token() {
        assert_eq!(
            escape("[fg:red]([bg:blue](inner))"),
            "\\[fg:red\\]\\(\\[bg:blue\\]\\(inner\\)\\)"
        );
    }

    #[test]
    fn kitchen_sink() {
        assert_eq!(
            escape("leading [fg:red](one [bg:blue](two [deco:dim](three) two) one) trailing"),
            "leading \\[fg:red\\]\\(one \\[bg:blue\\]\\(two \\[deco:dim\\]\\(three\\) two\\) one\\) trailing"
        );
    }

    #[test]
    fn parse_unescaped_open_parenthesis_in_plaintext() {
        assert_eq!(escape("before ( after"), "before \\( after");
    }

    #[test]
    fn parse_unescaped_close_parenthesis_in_plaintext() {
        assert_eq!(escape("before ) after"), "before \\) after");
    }

    #[test]
    fn parse_unescaped_open_square_bracket_in_plaintext() {
        assert_eq!(escape("before [ after"), "before \\[ after");
    }

    #[test]
    fn parse_unescaped_close_square_bracket_in_plaintext() {
        assert_eq!(escape("before ] after"), "before \\] after");
    }

    #[test]
    fn parse_unescaped_open_parenthesis_in_token() {
        assert_eq!(
            escape("[fg:red](before ( after)"),
            "\\[fg:red\\]\\(before \\( after\\)"
        );
    }

    #[test]
    fn parse_unescaped_close_parenthesis_in_token() {
        assert_eq!(
            escape("[fg:red](before ) after)"),
            "\\[fg:red\\]\\(before \\) after\\)"
        );
    }

    #[test]
    fn parse_unescaped_open_square_bracket_in_token() {
        assert_eq!(
            escape("[fg:red](before [ after)"),
            "\\[fg:red\\]\\(before \\[ after\\)"
        );
    }

    #[test]
    fn parse_unescaped_close_square_bracket_in_token() {
        assert_eq!(
            escape("[fg:red](before ] after)"),
            "\\[fg:red\\]\\(before \\] after\\)"
        );
    }

    #[test]
    fn parse_bad_escape_character() {
        assert_eq!(escape("before \\a after"), "before \\\\a after");
    }

    #[test]
    fn parse_bad_escape_character_in_token() {
        assert_eq!(
            escape("[fg:red](before \\a after)"),
            "\\[fg:red\\]\\(before \\\\a after\\)"
        );
    }
}
