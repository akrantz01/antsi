use super::{content::content, style::style, Parser};
use crate::ast::Token;

/// Parse a segment of text with styling
pub(crate) fn markup(p: &mut Parser) -> Option<Token> {
    Some(Token::Styled {
        style: style(p)?,
        content: content(p)?.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::{markup, Parser};

    #[test]
    fn foreground_no_content() {
        assert_parse_snapshot!(markup; "[fg:red]()");
    }

    #[test]
    fn background_no_content() {
        assert_parse_snapshot!(markup; "[bg:blue]()");
    }

    #[test]
    fn decoration_single_style_no_content() {
        assert_parse_snapshot!(markup; "[deco:dim]()");
    }

    #[test]
    fn decoration_multiple_styles_no_content() {
        assert_parse_snapshot!(markup; "[deco:dim,italic]()");
    }

    #[test]
    fn lowercase_alphabetic_content() {
        assert_parse_snapshot!(markup; "[fg:red](hello)");
    }

    #[test]
    fn uppercase_alphabetic_content() {
        assert_parse_snapshot!(markup; "[fg:red](HELLO)");
    }

    #[test]
    fn mixed_alphabetic_content() {
        assert_parse_snapshot!(markup; "[fg:red](hElLo)");
    }

    #[test]
    fn numeric_content() {
        assert_parse_snapshot!(markup; "[fg:red](12345)");
    }

    #[test]
    fn special_character_content() {
        assert_parse_snapshot!(markup; "[fg:red](!@#$%^)");
    }

    #[test]
    fn escaped_character_content() {
        assert_parse_snapshot!(markup; "[fg:red](\\(\\[\\]\\))");
    }

    #[test]
    fn nested_token() {
        assert_parse_snapshot!(markup; "[fg:red]([bg:blue](inner))");
    }

    #[test]
    fn nested_token_with_leading_content() {
        assert_parse_snapshot!(markup; "[fg:red](leading [bg:blue](inner))");
    }

    #[test]
    fn nested_token_with_trailing_content() {
        assert_parse_snapshot!(markup; "[fg:red]([bg:blue](inner) trailing)");
    }

    #[test]
    fn nested_token_with_leading_and_trailing_content() {
        assert_parse_snapshot!(markup; "[fg:red](leading [bg:blue](inner) trailing)");
    }
}
