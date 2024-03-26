use crate::ast::Tokens;
use crate::{
    ast::Token,
    lexer::{Lexeme, Lexer, SyntaxKind},
};
use std::iter::Peekable;
use text_size::TextRange;

mod content;
mod markup;
mod style;
mod text;

/// Convert a piece of text, potentially containing styled markup, to a sequence of tokens
pub struct Parser<'source> {
    lexer: Peekable<Lexer<'source>>,
    errors: Vec<ParseError>,
}

impl<'source> Parser<'source> {
    pub fn new(input: &'source str) -> Self {
        Self {
            lexer: Lexer::new(input).peekable(),
            errors: Vec::new(),
        }
    }

    /// Perform the parsing operation
    pub fn parse(mut self) -> (Vec<Token>, Vec<ParseError>) {
        let mut tokens = Tokens::default();

        loop {
            tokens.extend(text::text(&mut self).unwrap_or_default());

            if let Some(lexeme) = self.peek() {
                match dbg!(lexeme) {
                    SyntaxKind::ParenthesisOpen => {
                        self.error(ParseErrorReason::UnescapedControlCharacter('('))
                    }
                    SyntaxKind::ParenthesisClose => {
                        self.error(ParseErrorReason::UnescapedControlCharacter(')'))
                    }
                    SyntaxKind::SquareBracketOpen => {
                        self.error(ParseErrorReason::UnescapedControlCharacter('['))
                    }
                    SyntaxKind::SquareBracketClose => {
                        self.error(ParseErrorReason::UnescapedControlCharacter(']'))
                    }
                    _ => self.error(ParseErrorReason::Expected(vec![SyntaxKind::Eof])),
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

    /// Check if the parser is currently at one of the given syntax items
    pub(crate) fn at_one_of(&mut self, set: &[SyntaxKind]) -> bool {
        self.peek().map_or(false, |k| set.contains(&k))
    }

    /// Check if we're at the end of the token stream
    pub(crate) fn at_end(&mut self) -> bool {
        self.peek().is_none()
    }

    /// Expect a syntax item, emitting an error if it isn't present
    pub(crate) fn expect(&mut self, kind: SyntaxKind) -> Option<Lexeme> {
        if self.at(kind) {
            Some(self.bump())
        } else {
            self.error(ParseErrorReason::Expected(vec![kind]));
            None
        }
    }

    /// Report an error during parsing
    pub(crate) fn error(&mut self, reason: ParseErrorReason) {
        let (span, at) = match self.peek_lexeme() {
            Some(lexeme) => (Some(lexeme.span), lexeme.kind),
            None => (None, SyntaxKind::Eof),
        };

        self.errors.push(ParseError { span, at, reason })
    }
}

/// An error that occurred while parsing
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct ParseError {
    span: Option<TextRange>,
    at: SyntaxKind,
    reason: ParseErrorReason,
}

/// The reason for the parsing failure
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum ParseErrorReason {
    /// Expected a token, but found something else
    Expected(Vec<SyntaxKind>),
    /// Encountered an escape sequence that is not valid
    UnknownEscapeSequence(char),
    /// Encountered an unescaped control character
    UnescapedControlCharacter(char),
}

#[cfg(test)]
mod tests {
    use super::Parser;
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
