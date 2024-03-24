use crate::{
    ast::{Token, Tokens},
    lexer::{Lexeme, Lexer, SyntaxKind},
};
use std::iter::Peekable;

/// Convert a piece of text, potentially containing styled markup, to a sequence of tokens
pub struct Parser<'source> {
    lexer: Peekable<Lexer<'source>>,
    pub(crate) tokens: Tokens,
}

impl<'source> Parser<'source> {
    pub fn new(input: &'source str) -> Self {
        Self {
            lexer: Lexer::new(input).peekable(),
            tokens: Tokens::default(),
        }
    }

    /// Perform the parsing operation
    pub fn parse(mut self) -> Vec<Token> {
        match self.peek() {
            Some(SyntaxKind::Text) => {
                let lexeme = self.lexer.next().unwrap();
                self.tokens.push_str(lexeme.text);
            }
            None => {}
            _ => todo!(),
        }

        self.tokens.into()
    }

    /// Get the next syntax item from the lexer without consuming it
    pub(crate) fn peek(&mut self) -> Option<SyntaxKind> {
        self.lexer.peek().map(|lexeme| lexeme.kind)
    }

    /// Pop the next syntax item from the lexer
    pub(crate) fn bump(&mut self) -> Option<Lexeme> {
        self.lexer.next()
    }

    /// Check if the parser is currently at the given syntax item
    pub(crate) fn at(&mut self, kind: SyntaxKind) -> bool {
        self.peek() == Some(kind)
    }

    /// Check if the parser is currently at one of the given syntax items
    pub(crate) fn at_one_of(&mut self, set: &[SyntaxKind]) -> bool {
        self.peek().map_or(false, |k| set.contains(&k))
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;

    #[test]
    fn parse_empty() {
        let result = Parser::new("").parse();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn parse_text() {
        let result =
            Parser::new("this some text with wh ite\nspa\tce and numb3r5 and $ymb@l$ and CAPITALS")
                .parse();
        insta::assert_debug_snapshot!(result);
    }
}
