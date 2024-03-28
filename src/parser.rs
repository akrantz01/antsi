use crate::{
    ast::{Token, Tokens},
    error::{Error, Reason},
    lexer::{Lexeme, Lexer, SyntaxKind},
};
use std::iter::Peekable;

mod content;
mod markup;
mod style;
mod text;

/// Convert a piece of text, potentially containing styled markup, to a sequence of tokens
pub struct Parser<'source> {
    lexer: Peekable<Lexer<'source>>,
    errors: Vec<Error>,
}

impl<'source> Parser<'source> {
    pub fn new(input: &'source str) -> Self {
        Self {
            lexer: Lexer::new(input).peekable(),
            errors: Vec::new(),
        }
    }

    /// Perform the parsing operation
    pub fn parse(mut self) -> (Vec<Token>, Vec<Error>) {
        let mut tokens = Tokens::default();

        loop {
            tokens.extend(text::text(&mut self).unwrap_or_default());

            if let Some(lexeme) = self.peek() {
                match lexeme {
                    SyntaxKind::ParenthesisOpen => {
                        self.error(Reason::UnescapedControlCharacter('('))
                    }
                    SyntaxKind::ParenthesisClose => {
                        self.error(Reason::UnescapedControlCharacter(')'))
                    }
                    SyntaxKind::SquareBracketOpen => {
                        self.error(Reason::UnescapedControlCharacter('['))
                    }
                    SyntaxKind::SquareBracketClose => {
                        self.error(Reason::UnescapedControlCharacter(']'))
                    }
                    _ => self.error(Reason::Expected(vec![SyntaxKind::Eof])),
                }

                self.bump();
            } else {
                break;
            }
        }

        (tokens.into(), self.errors)
    }

    /// Get the next syntax item from the lexer without consuming it
    pub(crate) fn peek(&mut self) -> Option<SyntaxKind> {
        self.lexer.peek().map(|lexeme| lexeme.kind)
    }

    /// Get the next lexeme from the lexer without consuming it
    pub(crate) fn peek_lexeme(&mut self) -> Option<&Lexeme<'_>> {
        self.lexer.peek()
    }

    /// Pop the next syntax item from the lexer
    pub(crate) fn bump(&mut self) -> Lexeme {
        self.lexer.next().expect("missing token")
    }

    /// Check if the parser is currently at the given syntax item
    pub(crate) fn at(&mut self, kind: SyntaxKind) -> bool {
        self.peek() == Some(kind)
    }

    /// Expect a syntax item, emitting an error if it isn't present
    pub(crate) fn expect(&mut self, kind: SyntaxKind) -> Option<Lexeme> {
        if self.at(kind) {
            Some(self.bump())
        } else {
            self.error(Reason::Expected(vec![kind]));
            None
        }
    }

    /// Report an error during parsing
    pub(crate) fn error(&mut self, reason: Reason) {
        let (span, at) = match self.peek_lexeme() {
            Some(lexeme) => (Some(lexeme.span), lexeme.kind),
            None => (None, SyntaxKind::Eof),
        };

        self.errors.push(Error { span, at, reason })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Token;

    macro_rules! with_source {
        (
            $source:literal,
            $( { $( $name:ident => $value:expr ),+ $(,)? } , )?
            |$result:ident, $errors:ident| $actions:expr
        ) => {
            insta::with_settings!({
                description => $source,
                omit_expression => true,
                $( $( $name => $value, )+ )?
            }, {
                let ($result, $errors) = $crate::parser::Parser::new($source).parse();
                $actions;
            })
        };
    }

    macro_rules! assert_snapshot {
        ( { $( $name:ident => $value:expr ),+ $(,)? }, $expr:expr ) => {
            insta::with_settings!({
                $( $name => $value, )+
            }, {
                insta::assert_debug_snapshot!($expr);
            });
        };
        ($expr:expr) => {
            insta::assert_debug_snapshot!($expr);
        }
    }

    #[test]
    fn parse_empty() {
        with_source!("", |result, errors| {
            assert_eq!(result, vec![]);
            assert_eq!(errors, vec![]);
        });
    }

    #[test]
    fn parse_text() {
        with_source!(
            "this some text with wh ite\nspa\tce and numb3r5 and $ymb@l$ and CAPITALS",
            |result, errors| {
                assert_eq!(errors, vec![]);
                assert_snapshot!(result);
            }
        );
    }

    #[test]
    fn lowercase_alphabetic() {
        with_source!("abcdef", |result, errors| {
            assert_eq!(result, vec![Token::Content(String::from("abcdef"))]);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn uppercase_alphabetic() {
        with_source!("ABCDEF", |result, errors| {
            assert_eq!(result, vec![Token::Content(String::from("ABCDEF"))]);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn mixed_case_alphabetic() {
        with_source!("aBcDeF", |result, errors| {
            assert_eq!(result, vec![Token::Content(String::from("aBcDeF"))]);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn numeric() {
        with_source!("123456", |result, errors| {
            assert_eq!(result, vec![Token::Content(String::from("123456"))]);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn lowercase_alphanumeric() {
        with_source!("abc123", |result, errors| {
            assert_eq!(result, vec![Token::Content(String::from("abc123"))]);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn uppercase_alphanumeric() {
        with_source!("ABC123", |result, errors| {
            assert_eq!(result, vec![Token::Content(String::from("ABC123"))]);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn mixed_case_alphanumeric() {
        with_source!("AbCd1234", |result, errors| {
            assert_eq!(result, vec![Token::Content(String::from("AbCd1234"))]);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn special_characters() {
        with_source!("!@#$%^", |result, errors| {
            assert_eq!(result, vec![Token::Content(String::from("!@#$%^"))]);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn mixed_characters() {
        with_source!("ABCdef123!@#", |result, errors| {
            assert_eq!(result, vec![Token::Content(String::from("ABCdef123!@#"))]);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn escaped_characters() {
        with_source!("\\(\\)\\[\\]", |result, errors| {
            assert_eq!(result, vec![Token::Content(String::from("()[]"))]);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn escaped_whitespace() {
        with_source!("\\ \n\t", |result, errors| {
            assert_eq!(result, vec![]);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn empty_token() {
        with_source!("[fg:red]()", |result, errors| {
            assert_snapshot!(result);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn token_with_foreground() {
        with_source!("[fg:red](inner)", |result, errors| {
            assert_snapshot!(result);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn token_with_background() {
        with_source!("[bg:blue](inner)", |result, errors| {
            assert_snapshot!(result);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn token_with_single_decoration() {
        with_source!("[deco:dim](inner)", |result, errors| {
            assert_snapshot!(result);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn token_with_multiple_decorations() {
        with_source!("[deco:dim,italic](inner)", |result, errors| {
            assert_snapshot!(result);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn token_with_multiple_styles() {
        with_source!(
            "[deco:dim,italic;fg:red;bg:blue](inner)",
            |result, errors| {
                assert_snapshot!(result);
                assert!(errors.is_empty());
            }
        );
    }

    #[test]
    fn token_with_leading_content() {
        with_source!("leading [fg:red](content)", |result, errors| {
            assert_snapshot!(result);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn token_with_trailing_content() {
        with_source!("[fg:red](content) trailing", |result, errors| {
            assert_snapshot!(result);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn token_with_leading_and_trailing_content() {
        with_source!("leading [fg:red](content) trailing", |result, errors| {
            assert_snapshot!(result);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn nested_token() {
        with_source!("[fg:red]([bg:blue](inner))", |result, errors| {
            assert_snapshot!(result);
            assert!(errors.is_empty());
        });
    }

    #[test]
    fn kitchen_sink() {
        with_source!(
            "leading [fg:red](one [bg:blue](two [deco:dim](three) two) one) trailing",
            |result, errors| {
                assert_snapshot!(result);
                assert!(errors.is_empty());
            }
        );
    }

    #[test]
    fn parse_unescaped_open_parenthesis_in_plaintext() {
        with_source!("before ( after", |result, errors| {
            assert_snapshot!({ snapshot_suffix => "result" }, result);
            assert_snapshot!({ snapshot_suffix => "errors" }, errors);
        });
    }

    #[test]
    fn parse_unescaped_close_parenthesis_in_plaintext() {
        with_source!("before ) after", |result, errors| {
            assert_snapshot!({ snapshot_suffix => "result" }, result);
            assert_snapshot!({ snapshot_suffix => "errors" }, errors);
        });
    }

    #[test]
    fn parse_unescaped_open_square_bracket_in_plaintext() {
        with_source!("before [ after", |result, errors| {
            assert_eq!(result, vec![]);
            assert_snapshot!(errors);
        });
    }

    #[test]
    fn parse_unescaped_close_square_bracket_in_plaintext() {
        with_source!("before ] after", |result, errors| {
            assert_snapshot!({ snapshot_suffix => "result" }, result);
            assert_snapshot!({ snapshot_suffix => "errors" }, errors);
        });
    }

    #[test]
    fn parse_unescaped_open_parenthesis_in_token() {
        with_source!("[fg:red](before ( after)", |result, errors| {
            assert_snapshot!({ snapshot_suffix => "result" }, result);
            assert_snapshot!({ snapshot_suffix => "errors" }, errors);
        });
    }

    #[test]
    fn parse_unescaped_close_parenthesis_in_token() {
        with_source!("[fg:red](before ) after)", |result, errors| {
            assert_snapshot!({ snapshot_suffix => "result" }, result);
            assert_snapshot!({ snapshot_suffix => "errors" }, errors);
        });
    }

    #[test]
    fn parse_unescaped_open_square_bracket_in_token() {
        with_source!("[fg:red](before [ after)", |result, errors| {
            assert_eq!(result, vec![]);
            assert_snapshot!(errors);
        });
    }

    #[test]
    fn parse_unescaped_close_square_bracket_in_token() {
        with_source!("[fg:red](before ] after)", |result, errors| {
            assert_snapshot!({ snapshot_suffix => "result" }, result);
            assert_snapshot!({ snapshot_suffix => "errors" }, errors);
        });
    }

    #[test]
    fn parse_bad_escape_character() {
        with_source!("before \\a after", |result, errors| {
            assert_snapshot!({ snapshot_suffix => "result" }, result);
            assert_snapshot!({ snapshot_suffix => "errors" }, errors);
        });
    }

    #[test]
    fn parse_bad_escape_character_in_token() {
        with_source!("[fg:red](before \\a after)", |result, errors| {
            assert_snapshot!({ snapshot_suffix => "result" }, result);
            assert_snapshot!({ snapshot_suffix => "errors" }, errors);
        });
    }
}
