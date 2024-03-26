use crate::lexer::SyntaxKind;
use text_size::TextRange;

/// An error that occurred while parsing
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Error {
    pub span: Option<TextRange>,
    pub at: SyntaxKind,
    pub reason: Reason,
}

/// The reason for the parsing failure
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum Reason {
    /// Expected a token, but found something else
    Expected(Vec<SyntaxKind>),
    /// Encountered an escape sequence that is not valid
    UnknownEscapeSequence(char),
    /// Encountered an unescaped control character
    UnescapedControlCharacter(char),
}
