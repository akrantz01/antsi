use super::{markup::markup, Parser};
use crate::{ast::Tokens, error::Reason, lexer::SyntaxKind};

/// Parse a piece of text that may content styled markup
pub(crate) fn text(p: &mut Parser) -> Option<Tokens> {
    let mut tokens = Tokens::default();

    loop {
        match p.peek() {
            Some(
                SyntaxKind::ParenthesisClose
                | SyntaxKind::ParenthesisOpen
                | SyntaxKind::SquareBracketClose,
            ) => break,
            Some(SyntaxKind::SquareBracketOpen) => {
                let styled = markup(p)?;
                tokens.push(styled);
            }
            Some(SyntaxKind::EscapeWhitespace) => {
                p.bump();
            }
            Some(SyntaxKind::EscapeCharacter) => {
                let lexeme = p.peek_lexeme().unwrap();

                assert_eq!(lexeme.text.len(), 2);
                let character = lexeme.text.chars().nth(1).unwrap();
                match character {
                    '\\' | '(' | ')' | '[' | ']' => {
                        tokens.push_char(character);
                    }
                    _ => {
                        p.error(Reason::UnknownEscapeSequence(character));
                    }
                }

                p.bump();
            }
            Some(SyntaxKind::Eof | SyntaxKind::Unknown) => unreachable!(),
            Some(_) => {
                let lexeme = p.bump();
                tokens.push_str(lexeme.text);
            }
            None => break,
        }
    }

    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::{text, Parser};
    use crate::{
        ast::{Token, Tokens},
        error::{Error, Reason},
        lexer::SyntaxKind,
    };

    #[test]
    fn empty() {
        let mut parser = Parser::new("");
        assert_eq!(text(&mut parser), Some(Tokens::from(vec![])));
    }

    #[test]
    fn stops_consuming_at_open_parenthesis() {
        let mut parser = Parser::new("before(after");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("before"))]))
        );
        assert!(parser.at(SyntaxKind::ParenthesisOpen));
    }

    #[test]
    fn stops_consuming_at_close_parenthesis() {
        let mut parser = Parser::new("before)after");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("before"))]))
        );
        assert!(parser.at(SyntaxKind::ParenthesisClose));
    }

    #[test]
    fn stops_consuming_at_close_square_bracket() {
        let mut parser = Parser::new("before]after");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("before"))]))
        );
        assert!(parser.at(SyntaxKind::SquareBracketClose));
    }

    #[test]
    fn lowercase_alphabetic() {
        let mut parser = Parser::new("abcdefghijklmnopqrstuvwxyz");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "abcdefghijklmnopqrstuvwxyz"
            ))]))
        )
    }

    #[test]
    fn uppercase_alphabetic() {
        let mut parser = Parser::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            ))]))
        )
    }

    #[test]
    fn mixed_case_alphabetic() {
        let mut parser = Parser::new("AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYuZz");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYuZz"
            ))]))
        )
    }

    #[test]
    fn special_characters() {
        let mut parser = Parser::new("~!@#$%^&*-=_+~");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "~!@#$%^&*-=_+~"
            ))]))
        )
    }

    #[test]
    fn whitespace() {
        let mut parser = Parser::new(" \n\t\r");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(" \n\t\r"))]))
        )
    }

    #[test]
    fn matching_color() {
        let mut parser = Parser::new("black");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("black"))]))
        );
    }

    #[test]
    fn matching_bright_color() {
        let mut parser = Parser::new("bright-blue");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "bright-blue"
            ))]))
        );
    }

    #[test]
    fn matching_default_color() {
        let mut parser = Parser::new("default");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("default"))]))
        );
    }

    #[test]
    fn matching_decoration() {
        let mut parser = Parser::new("fast-blink");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "fast-blink"
            ))]))
        )
    }

    #[test]
    fn containing_colon() {
        let mut parser = Parser::new(":");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(":"))]))
        )
    }

    #[test]
    fn containing_semicolon() {
        let mut parser = Parser::new(";");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(";"))]))
        )
    }

    #[test]
    fn containing_comma() {
        let mut parser = Parser::new(",");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(","))]))
        )
    }

    #[test]
    fn containing_foreground_specifier() {
        let mut parser = Parser::new("fg");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("fg"))]))
        );
    }

    #[test]
    fn containing_background_specifier() {
        let mut parser = Parser::new("bg");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("bg"))]))
        );
    }

    #[test]
    fn containing_decoration_specifier() {
        let mut parser = Parser::new("deco");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("deco"))]))
        );
    }

    #[test]
    fn escaped_backslash() {
        let mut parser = Parser::new("\\\\");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("\\"))]))
        )
    }

    #[test]
    fn escaped_open_square_bracket() {
        let mut parser = Parser::new("\\[");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("["))]))
        )
    }

    #[test]
    fn escaped_close_square_bracket() {
        let mut parser = Parser::new("\\]");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("]"))]))
        )
    }

    #[test]
    fn escaped_open_parenthesis() {
        let mut parser = Parser::new("\\(");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("("))]))
        )
    }

    #[test]
    fn escaped_close_parenthesis() {
        let mut parser = Parser::new("\\)");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(")"))]))
        )
    }

    #[test]
    fn escaped_whitespace() {
        let mut parser = Parser::new("\\ \n\t\r");
        assert_eq!(text(&mut parser), Some(Tokens::from(vec![])));
    }

    #[test]
    fn multiple_distinct_tokens() {
        let mut parser = Parser::new("some plaintext \\(ascii\\] \\\n\n :+1:");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "some plaintext (ascii] :+1:"
            ))]))
        );
    }

    #[test]
    fn mixed_characters_and_escape_characters() {
        let mut parser = Parser::new("abc\\(DEF\\)12\\   34\\[!@#$\\]");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "abc(DEF)1234[!@#$]"
            ))]))
        );
    }

    #[test]
    fn empty_token() {
        assert_parse_snapshot!(text; "[fg:red]()");
    }

    #[test]
    fn token_with_foreground() {
        assert_parse_snapshot!(text; "[fg:red](inner)");
    }

    #[test]
    fn token_with_background() {
        assert_parse_snapshot!(text; "[bg:blue](inner)");
    }

    #[test]
    fn token_with_single_decoration() {
        assert_parse_snapshot!(text; "[deco:dim](inner)");
    }

    #[test]
    fn token_with_multiple_decorations() {
        assert_parse_snapshot!(text; "[deco:dim,italic](inner)");
    }

    #[test]
    fn token_with_multiple_styles() {
        assert_parse_snapshot!(text; "[deco:dim,italic;fg:red;bg:blue](inner)");
    }

    #[test]
    fn token_with_leading_content() {
        assert_parse_snapshot!(text; "leading [fg:red](content)");
    }

    #[test]
    fn token_with_trailing_content() {
        assert_parse_snapshot!(text; "[fg:red](content) trailing");
    }

    #[test]
    fn token_with_leading_and_trailing_content() {
        assert_parse_snapshot!(text; "leading [fg:red](content) trailing");
    }

    #[test]
    fn nested_token() {
        assert_parse_snapshot!(text; "[fg:red]([bg:blue](inner))");
    }

    #[test]
    fn kitchen_sink() {
        assert_parse_snapshot!(text; "leading [fg:red](one [bg:blue](two [deco:dim](three) two) one) trailing");
    }

    #[test]
    fn unescaped_open_parenthesis_in_plaintext() {
        let mut parser = Parser::new("before ( after");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("before "))]))
        );
        assert_eq!(parser.peek(), Some(SyntaxKind::ParenthesisOpen));
    }

    #[test]
    fn unescaped_close_parenthesis_in_plaintext() {
        let mut parser = Parser::new("before ) after");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("before "))]))
        );
        assert_eq!(parser.peek(), Some(SyntaxKind::ParenthesisClose));
    }

    #[test]
    fn unescaped_open_square_bracket_in_plaintext() {
        let mut parser = Parser::new("before [ after");
        assert_eq!(text(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(8..14)),
                at: SyntaxKind::Text,
                reason: Reason::Expected(vec![
                    SyntaxKind::ForegroundSpecifier,
                    SyntaxKind::BackgroundSpecifier,
                    SyntaxKind::DecorationSpecifier,
                ])
            }]
        );
    }

    #[test]
    fn unescaped_close_square_bracket_in_plaintext() {
        let mut parser = Parser::new("before ] after");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("before "))]))
        );
        assert_eq!(parser.peek(), Some(SyntaxKind::SquareBracketClose));
    }

    #[test]
    fn unescaped_open_parenthesis_in_token() {
        let mut parser = Parser::new("[fg:red](before ( after)");
        assert_eq!(text(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(16..17)),
                at: SyntaxKind::ParenthesisOpen,
                reason: Reason::Expected(vec![SyntaxKind::ParenthesisClose])
            }]
        );
    }

    #[test]
    fn unescaped_close_parenthesis_in_token() {
        let mut parser = Parser::new("[fg:red](before ) after)");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![
                Token::Styled {
                    content: vec![Token::Content(String::from("before "))],
                    style: style!(fg: Red;)
                },
                Token::Content(String::from(" after"))
            ]))
        );
        assert_eq!(parser.peek(), Some(SyntaxKind::ParenthesisClose));
    }

    #[test]
    fn unescaped_open_square_bracket_in_token() {
        let mut parser = Parser::new("[fg:red](before [ after)");
        assert_eq!(text(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(17..23)),
                at: SyntaxKind::Text,
                reason: Reason::Expected(vec![
                    SyntaxKind::ForegroundSpecifier,
                    SyntaxKind::BackgroundSpecifier,
                    SyntaxKind::DecorationSpecifier,
                ])
            }]
        );
    }

    #[test]
    fn unescaped_close_square_bracket_in_token() {
        let mut parser = Parser::new("[fg:red](before ] after)");
        assert_eq!(text(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(16..17)),
                at: SyntaxKind::SquareBracketClose,
                reason: Reason::Expected(vec![SyntaxKind::ParenthesisClose])
            }]
        );
    }

    #[test]
    fn token_empty_specifier() {
        let mut parser = Parser::new("[]()");
        assert_eq!(text(&mut parser), None);
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
    fn token_unclosed_specifier() {
        let mut parser = Parser::new("[fg:red");
        assert_eq!(text(&mut parser), None);
        assert_eq!(
            parser.errors,
            vec![Error {
                span: None,
                at: SyntaxKind::Eof,
                reason: Reason::Expected(vec![SyntaxKind::SquareBracketClose])
            }]
        );
    }

    #[test]
    fn token_unclosed_content() {
        let mut parser = Parser::new("[fg:red](test");
        assert_eq!(text(&mut parser), None);
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
    fn invalid_escape_character() {
        let mut parser = Parser::new("\\a");
        assert_eq!(text(&mut parser), Some(Tokens::from(vec![])));
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(0..2)),
                at: SyntaxKind::EscapeCharacter,
                reason: Reason::UnknownEscapeSequence('a')
            }]
        );
    }

    #[test]
    fn token_invalid_escape_character() {
        let mut parser = Parser::new("[fg:red](\\a)");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Styled {
                style: style!(fg: Red;),
                content: vec![]
            }]))
        );
        assert_eq!(
            parser.errors,
            vec![Error {
                span: Some(span!(9..11)),
                at: SyntaxKind::EscapeCharacter,
                reason: Reason::UnknownEscapeSequence('a')
            }]
        );
    }
}
