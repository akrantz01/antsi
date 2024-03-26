use super::{text::text, Parser};
use crate::{ast::Tokens, lexer::SyntaxKind};

/// Parse a piece of styled content
pub(crate) fn content(p: &mut Parser) -> Option<Tokens> {
    p.expect(SyntaxKind::ParenthesisOpen)?;

    let tokens = text(p)?;

    p.expect(SyntaxKind::ParenthesisClose)?;

    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::{content, Parser};
    use crate::{
        ast::{Token, Tokens},
        error::{Error, Reason},
        lexer::SyntaxKind,
    };

    #[test]
    fn empty() {
        let mut parser = Parser::new("()");
        assert_eq!(content(&mut parser), Some(Tokens::from(vec![])));
    }

    #[test]
    fn lowercase_alphabetic() {
        let mut parser = Parser::new("(abcdefghijklmnopqrstuvwxyz)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "abcdefghijklmnopqrstuvwxyz"
            ))]))
        )
    }

    #[test]
    fn uppercase_alphabetic() {
        let mut parser = Parser::new("(ABCDEFGHIJKLMNOPQRSTUVWXYZ)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            ))]))
        )
    }

    #[test]
    fn mixed_case_alphabetic() {
        let mut parser = Parser::new("(AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYuZz)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYuZz"
            ))]))
        )
    }

    #[test]
    fn special_characters() {
        let mut parser = Parser::new("(~!@#$%^&*-=_+~)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "~!@#$%^&*-=_+~"
            ))]))
        )
    }

    #[test]
    fn whitespace() {
        let mut parser = Parser::new("( \n\t\r)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(" \n\t\r"))]))
        )
    }

    #[test]
    fn matching_color() {
        let mut parser = Parser::new("(black)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("black"))]))
        );
    }

    #[test]
    fn matching_bright_color() {
        let mut parser = Parser::new("(bright-blue)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "bright-blue"
            ))]))
        );
    }

    #[test]
    fn matching_default_color() {
        let mut parser = Parser::new("(default)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("default"))]))
        );
    }

    #[test]
    fn matching_decoration() {
        let mut parser = Parser::new("(fast-blink)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "fast-blink"
            ))]))
        )
    }

    #[test]
    fn containing_colon() {
        let mut parser = Parser::new("(:)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(":"))]))
        )
    }

    #[test]
    fn containing_semicolon() {
        let mut parser = Parser::new("(;)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(";"))]))
        )
    }

    #[test]
    fn containing_comma() {
        let mut parser = Parser::new("(,)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(","))]))
        )
    }

    #[test]
    fn containing_foreground_specifier() {
        let mut parser = Parser::new("(fg)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("fg"))]))
        );
    }

    #[test]
    fn containing_background_specifier() {
        let mut parser = Parser::new("(bg)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("bg"))]))
        );
    }

    #[test]
    fn containing_decoration_specifier() {
        let mut parser = Parser::new("(deco)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("deco"))]))
        );
    }

    #[test]
    fn escaped_backslash() {
        let mut parser = Parser::new("(\\\\)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("\\"))]))
        )
    }

    #[test]
    fn escaped_open_square_bracket() {
        let mut parser = Parser::new("(\\[)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("["))]))
        )
    }

    #[test]
    fn escaped_close_square_bracket() {
        let mut parser = Parser::new("(\\])");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("]"))]))
        )
    }

    #[test]
    fn escaped_open_parenthesis() {
        let mut parser = Parser::new("(\\()");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("("))]))
        )
    }

    #[test]
    fn escaped_close_parenthesis() {
        let mut parser = Parser::new("(\\))");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(")"))]))
        )
    }

    #[test]
    fn escaped_whitespace() {
        let mut parser = Parser::new("(\\ \n\t\r)");
        assert_eq!(content(&mut parser), Some(Tokens::from(vec![])));
    }

    #[test]
    fn missing_closing_parenthesis() {
        let mut parser = Parser::new("(test");
        assert_eq!(content(&mut parser), None);
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
    fn unescaped_open_square_bracket() {
        let mut parser = Parser::new("([)");
        assert_eq!(content(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(2..3)),
                at: SyntaxKind::ParenthesisClose,
                reason: Reason::Expected(vec![
                    SyntaxKind::ForegroundSpecifier,
                    SyntaxKind::BackgroundSpecifier,
                    SyntaxKind::DecorationSpecifier
                ])
            }]
        );
    }

    #[test]
    fn unescaped_close_square_bracket() {
        let mut parser = Parser::new("(])");
        assert_eq!(content(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(1..2)),
                at: SyntaxKind::SquareBracketClose,
                reason: Reason::Expected(vec![SyntaxKind::ParenthesisClose])
            }]
        );
    }

    #[test]
    fn unescaped_open_parenthesis() {
        let mut parser = Parser::new("(()");
        assert_eq!(content(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(1..2)),
                at: SyntaxKind::ParenthesisOpen,
                reason: Reason::Expected(vec![SyntaxKind::ParenthesisClose])
            }]
        );
    }

    #[test]
    fn unescaped_close_parenthesis() {
        let mut parser = Parser::new("())");
        assert_eq!(content(&mut parser), Some(Tokens::from(vec![])));
        assert_eq!(parser.peek(), Some(SyntaxKind::ParenthesisClose));
    }
}
