use crate::lexer::SyntaxKind;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::{Error as CodespanError, SimpleFile},
    term::Config,
};
use std::io;
use termcolor::{Buffer, WriteColor};
use text_size::{TextLen, TextRange};

/// A report of all the issues found with a piece of text
#[derive(Clone, Debug)]
pub struct ErrorReport(Vec<Error>);

impl From<Vec<Error>> for ErrorReport {
    fn from(errors: Vec<Error>) -> Self {
        Self(errors)
    }
}

impl ErrorReport {
    /// Get the contained errors
    pub fn errors(&self) -> &[Error] {
        self.0.as_slice()
    }

    /// Emit the report to a string
    pub fn emit(&self, file: &str, source: &str, supports_color: bool) -> io::Result<String> {
        let mut buffer = if supports_color {
            Buffer::ansi()
        } else {
            Buffer::no_color()
        };

        self.emit_to(file, source, &mut buffer)?;

        Ok(String::from_utf8(buffer.into_inner()).expect("should be valid utf-8"))
    }

    /// Emit the report to the specified output
    pub fn emit_to(&self, file: &str, source: &str, output: &mut dyn WriteColor) -> io::Result<()> {
        let file = SimpleFile::new(file, source);
        let config = Config::default();

        let eof = {
            let length = source.text_len();
            TextRange::new(length, length)
        };

        for error in self.errors() {
            if let Err(err) = codespan_reporting::term::emit(
                output,
                &config,
                &file,
                &error.to_diagnostic((), eof),
            ) {
                match err {
                    CodespanError::Io(e) => return Err(e),
                    _ => panic!("reporting failed: {err:?}"),
                }
            }
        }

        Ok(())
    }
}

/// An error that occurred while parsing
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Error {
    pub span: Option<TextRange>,
    pub at: SyntaxKind,
    pub reason: Reason,
}

impl Error {
    /// Convert the error into a user-friendly diagnostic
    pub fn to_diagnostic<FileId>(&self, file: FileId, eof: TextRange) -> Diagnostic<FileId>
    where
        FileId: Copy,
    {
        let span = self.span.unwrap_or(eof);
        match &self.reason {
            Reason::Expected(tokens) => Diagnostic::error()
                .with_message("unexpected token encountered")
                .with_labels(vec![
                    Label::primary(file, span).with_message(format!("found {} token", self.at)),
                    Label::secondary(file, span).with_message({
                        let comma_separated = tokens.iter().map(SyntaxKind::name).enumerate().fold(
                            String::new(),
                            |mut acc, (i, name)| {
                                if i > 0 {
                                    acc.push_str(", ");
                                }

                                acc.push_str(name);
                                acc
                            },
                        );
                        format!("expected one of: {comma_separated}")
                    }),
                ]),
            Reason::UnknownEscapeSequence(character) => Diagnostic::error()
                .with_message("unknown escape sequence")
                .with_labels(vec![Label::primary(file, span)
                    .with_message(format!("unknown escaped character `{character}`"))])
                .with_notes(vec![String::from(
                    "valid escape sequences are: `\\\\`, `\\[`, `\\]`, `\\(`, `\\)`",
                )]),
            Reason::UnescapedControlCharacter(character) => Diagnostic::error()
                .with_message("unescaped control character")
                .with_labels(vec![
                    Label::primary(file, span).with_message(format!(
                        "found an unescaped `{character}` that needs to be escaped"
                    )),
                    Label::secondary(file, span)
                        .with_message(format!("use `\\{character}` to escape it")),
                ]),
        }
    }
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
