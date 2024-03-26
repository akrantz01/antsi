use crate::{
    ast::Token,
    lexer::{Lexeme, Lexer, SyntaxKind},
};
use std::{iter::Peekable, ops::Range};
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
        let tokens = text::text(&mut self).unwrap_or_default().into();

        if self.peek().is_some() {
            self.error(ParseErrorReason::Expected(vec![SyntaxKind::Eof]));
        }

        (tokens, self.errors)
    }

    /// Get the next syntax item from the lexer without consuming it
    pub(crate) fn peek(&mut self) -> Option<SyntaxKind> {
        self.lexer.peek().map(|lexeme| lexeme.kind)
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
        let (span, at) = match self.lexer.peek() {
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
}

#[cfg(test)]
mod tests {
    use super::Parser;

    #[test]
    fn parse_empty() {
        let (result, errors) = Parser::new("").parse();
        assert_eq!(result, vec![]);
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn parse_text() {
        let (result, errors) =
            Parser::new("this some text with wh ite\nspa\tce and numb3r5 and $ymb@l$ and CAPITALS")
                .parse();
        assert_eq!(errors, vec![]);
        insta::assert_debug_snapshot!(result);
    }
}
