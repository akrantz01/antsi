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
    use crate::{
        error::{Error, Reason},
        lexer::SyntaxKind,
    };

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

    #[test]
    fn empty_style_specifiers() {
        let mut parser = Parser::new("[](content)");
        assert_eq!(markup(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(1..2)),
                at: SyntaxKind::SquareBracketClose,
                reason: Reason::Expected(vec![
                    SyntaxKind::ForegroundSpecifier,
                    SyntaxKind::BackgroundSpecifier,
                    SyntaxKind::DecorationSpecifier
                ])
            }]
        );
    }

    #[test]
    fn missing_closing_square_bracket_on_style_specifiers() {
        let mut parser = Parser::new("[fg:red(content)");
        assert_eq!(markup(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(7..8)),
                at: SyntaxKind::ParenthesisOpen,
                reason: Reason::Expected(vec![SyntaxKind::SquareBracketClose])
            }]
        );
    }

    #[test]
    fn missing_open_parenthesis_for_content() {
        let mut parser = Parser::new("[fg:red]content)");
        assert_eq!(markup(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(8..15)),
                at: SyntaxKind::Text,
                reason: Reason::Expected(vec![SyntaxKind::ParenthesisOpen])
            }]
        );
    }

    #[test]
    fn missing_close_parenthesis_for_content() {
        let mut parser = Parser::new("[fg:red](content");
        assert_eq!(markup(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: None,
                at: SyntaxKind::Eof,
                reason: Reason::Expected(vec![SyntaxKind::ParenthesisClose])
            }]
        );
    }

    #[test]
    fn content_does_not_immediately_follow_style_specifiers() {
        let mut parser = Parser::new("[fg:red] (content)");
        assert_eq!(markup(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(8..9)),
                at: SyntaxKind::Text,
                reason: Reason::Expected(vec![SyntaxKind::ParenthesisOpen])
            }]
        );
    }
}
