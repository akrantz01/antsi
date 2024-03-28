use logos::Logos;
use std::{
    fmt::{Display, Formatter},
    ops::Range,
};
use text_size::{TextRange, TextSize};

pub(crate) struct Lexer<'source>(logos::Lexer<'source, SyntaxKind>);

impl<'source> Lexer<'source> {
    pub fn new(input: &'source str) -> Self {
        Self(SyntaxKind::lexer(input))
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Lexeme<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.0.next()?.unwrap_or(SyntaxKind::Unknown);
        let span = {
            let Range { start, end } = self.0.span();
            let start = TextSize::try_from(start).unwrap();
            let end = TextSize::try_from(end).unwrap();

            TextRange::new(start, end)
        };

        Some(Lexeme {
            span,
            kind,
            text: self.0.slice(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Logos, PartialEq)]
pub(crate) enum SyntaxKind {
    #[token("[")]
    SquareBracketOpen,

    #[token("]")]
    SquareBracketClose,

    #[token("(")]
    ParenthesisOpen,

    #[token(")")]
    ParenthesisClose,

    #[token(":", priority = 10)]
    Colon,

    #[token(";", priority = 10)]
    Semicolon,

    #[token(",", priority = 10)]
    Comma,

    #[token("fg", priority = 10, ignore(ascii_case))]
    ForegroundSpecifier,

    #[token("bg", priority = 10, ignore(ascii_case))]
    BackgroundSpecifier,

    #[token("deco", priority = 10, ignore(ascii_case))]
    DecorationSpecifier,

    #[regex(
        r#"(bright-)?(black|red|green|yellow|blue|magenta|cyan|white)"#,
        priority = 10,
        ignore(ascii_case)
    )]
    #[token("default", ignore(ascii_case))]
    Color,

    #[regex(
        r#"(bold|dim|faint|italic|underline|(fast|slow)-blink|blink-(fast|slow)|invert|reverse|hide|conceal|strike(-)?through)"#,
        priority = 10,
        ignore(ascii_case)
    )]
    Decoration,

    #[regex(r#"\\[^ \r\n\t]"#)]
    EscapeCharacter,

    #[regex(r#"\\[ \r\n\t]+"#)]
    EscapeWhitespace,

    // as a temporary fix until https://github.com/maciejhirsz/logos/issues/265 is resolved, the
    // tokens `:` `;` and `,` are considered stop characters for words
    #[regex(r#"[^\\\[\]():;,]+"#, priority = 2)]
    Text,

    Unknown,
    Eof,
}

impl SyntaxKind {
    /// Get the name of the lexeme
    pub fn name(&self) -> &'static str {
        match self {
            Self::SquareBracketOpen => "[",
            Self::SquareBracketClose => "]",
            Self::ParenthesisOpen => "(",
            Self::ParenthesisClose => ")",
            Self::Colon => ":",
            Self::Comma => ",",
            Self::Semicolon => ";",
            Self::ForegroundSpecifier => "foreground specifier",
            Self::BackgroundSpecifier => "background specifier",
            Self::DecorationSpecifier => "decoration specifier",
            Self::Color => "color",
            Self::Decoration => "decoration",
            Self::EscapeCharacter => "escape character",
            Self::EscapeWhitespace => "escape whitespace",
            Self::Text => "text",
            Self::Unknown => "unknown",
            Self::Eof => "end of file",
        }
    }
}

impl Display for SyntaxKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug)]
pub(crate) struct Lexeme<'source> {
    pub kind: SyntaxKind,
    pub text: &'source str,
    pub span: TextRange,
}

#[cfg(test)]
mod tests {
    use super::{Lexer, SyntaxKind};
    use text_size::TextSize;

    fn check(input: &str, kind: SyntaxKind) {
        let mut lexer = Lexer::new(input);

        let token = lexer.next().unwrap();
        assert_eq!(token.kind, kind);
        assert_eq!(token.text, input);
        assert_ne!(token.span.len(), TextSize::new(0));
    }

    #[test]
    fn square_bracket_open() {
        check("[", SyntaxKind::SquareBracketOpen);
    }

    #[test]
    fn square_bracket_close() {
        check("]", SyntaxKind::SquareBracketClose);
    }

    #[test]
    fn parenthesis_open() {
        check("(", SyntaxKind::ParenthesisOpen);
    }

    #[test]
    fn parenthesis_close() {
        check(")", SyntaxKind::ParenthesisClose);
    }

    #[test]
    fn colon() {
        check(":", SyntaxKind::Colon);
    }

    #[test]
    fn semicolon() {
        check(";", SyntaxKind::Semicolon);
    }

    #[test]
    fn comma() {
        check(",", SyntaxKind::Comma);
    }

    #[test]
    fn lowerascii_case_alphabetic_text() {
        check("abcdefghijklmnopqrstuvwxyz", SyntaxKind::Text);
    }

    #[test]
    fn upperascii_case_alphabetic_text() {
        check("ABCDEFGHIJKLMNOPQRSTUVWXYZ", SyntaxKind::Text);
    }

    #[test]
    fn mixed_ascii_case_alphabetic_text() {
        check(
            "AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz",
            SyntaxKind::Text,
        );
    }

    #[test]
    fn numeric_text() {
        check("1234567890", SyntaxKind::Text);
    }

    #[test]
    fn special_characters_text() {
        check("!@#$%^&*-_+=", SyntaxKind::Text);
    }

    #[test]
    fn foreground_specifier() {
        check("fg", SyntaxKind::ForegroundSpecifier);
    }

    #[test]
    fn background_specifier() {
        check("bg", SyntaxKind::BackgroundSpecifier);
    }

    #[test]
    fn decoration_specifier() {
        check("deco", SyntaxKind::DecorationSpecifier);
    }

    #[test]
    fn whitespace_text() {
        check("  \n\t", SyntaxKind::Text);
    }

    #[test]
    fn color_black() {
        check("black", SyntaxKind::Color);
    }

    #[test]
    fn color_red() {
        check("red", SyntaxKind::Color);
    }

    #[test]
    fn color_green() {
        check("green", SyntaxKind::Color);
    }

    #[test]
    fn color_yellow() {
        check("yellow", SyntaxKind::Color);
    }

    #[test]
    fn color_blue() {
        check("blue", SyntaxKind::Color);
    }

    #[test]
    fn color_magenta() {
        check("magenta", SyntaxKind::Color);
    }

    #[test]
    fn color_cyan() {
        check("cyan", SyntaxKind::Color);
    }

    #[test]
    fn color_white() {
        check("white", SyntaxKind::Color);
    }

    #[test]
    fn color_default() {
        check("default", SyntaxKind::Color);
    }

    #[test]
    fn color_bright_black() {
        check("bright-black", SyntaxKind::Color);
    }

    #[test]
    fn color_bright_red() {
        check("bright-red", SyntaxKind::Color);
    }

    #[test]
    fn color_bright_green() {
        check("bright-green", SyntaxKind::Color);
    }

    #[test]
    fn color_bright_yellow() {
        check("bright-yellow", SyntaxKind::Color);
    }

    #[test]
    fn color_bright_blue() {
        check("bright-blue", SyntaxKind::Color);
    }

    #[test]
    fn color_bright_magenta() {
        check("bright-magenta", SyntaxKind::Color);
    }

    #[test]
    fn color_bright_cyan() {
        check("bright-cyan", SyntaxKind::Color);
    }

    #[test]
    fn color_bright_white() {
        check("bright-white", SyntaxKind::Color);
    }

    #[test]
    fn decoration_bold() {
        check("bold", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_dim() {
        check("dim", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_faint() {
        check("faint", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_italic() {
        check("italic", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_underline() {
        check("underline", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_fast_blink() {
        check("fast-blink", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_slow_blink() {
        check("slow-blink", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_blink_fast() {
        check("blink-fast", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_blink_slow() {
        check("blink-slow", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_invert() {
        check("invert", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_reverse() {
        check("reverse", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_hide() {
        check("hide", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_conceal() {
        check("conceal", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_strikethrough() {
        check("strikethrough", SyntaxKind::Decoration);
    }

    #[test]
    fn decoration_strike_through() {
        check("strike-through", SyntaxKind::Decoration);
    }

    #[test]
    fn escape_character_backslash() {
        check("\\\\", SyntaxKind::EscapeCharacter);
    }

    #[test]
    fn escape_character_open_square_bracket() {
        check("\\[", SyntaxKind::EscapeCharacter);
    }

    #[test]
    fn escape_character_close_square_bracket() {
        check("\\]", SyntaxKind::EscapeCharacter);
    }

    #[test]
    fn escape_character_open_parenthesis() {
        check("\\(", SyntaxKind::EscapeCharacter);
    }

    #[test]
    fn escape_character_close_parenthesis() {
        check("\\)", SyntaxKind::EscapeCharacter);
    }

    #[test]
    fn escape_character_invalid() {
        check("\\a", SyntaxKind::EscapeCharacter);
        check("\\$", SyntaxKind::EscapeCharacter);
        check("\\Z", SyntaxKind::EscapeCharacter);
        check("\\4", SyntaxKind::EscapeCharacter);
    }

    #[test]
    fn escape_whitespace_single_space() {
        check("\\ ", SyntaxKind::EscapeWhitespace);
    }

    #[test]
    fn escape_whitespace_multiple_spaces() {
        check("\\     ", SyntaxKind::EscapeWhitespace);
    }

    #[test]
    fn escape_whitespace_single_carriage_return() {
        check("\\\r", SyntaxKind::EscapeWhitespace);
    }

    #[test]
    fn escape_whitespace_multiple_carriage_returns() {
        check("\\\r\r\r\r\r", SyntaxKind::EscapeWhitespace);
    }

    #[test]
    fn escape_whitespace_single_newline() {
        check("\\\n", SyntaxKind::EscapeWhitespace);
    }

    #[test]
    fn escape_whitespace_multiple_newlines() {
        check("\\\n\n\n\n\n", SyntaxKind::EscapeWhitespace);
    }

    #[test]
    fn escape_whitespace_single_tab() {
        check("\\\t", SyntaxKind::EscapeWhitespace);
    }

    #[test]
    fn escape_whitespace_multiple_tabs() {
        check("\\\t\t\t\t\t", SyntaxKind::EscapeWhitespace);
    }

    #[test]
    fn escape_whitespace_mixed() {
        check("\\ \t\r\n", SyntaxKind::EscapeWhitespace);
    }

    #[test]
    fn foreground_style_specifier() {
        let tokens = Lexer::new("fg:blue").collect::<Vec<_>>();
        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn background_style_specifier() {
        let tokens = Lexer::new("bg:magenta").collect::<Vec<_>>();
        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn single_decoration_style_specifier() {
        let tokens = Lexer::new("deco:bold").collect::<Vec<_>>();
        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn multiple_decoration_style_specifiers() {
        let tokens = Lexer::new("deco:bold,italic").collect::<Vec<_>>();
        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    fn many_tokens() {
        let tokens = Lexer::new(
            "leading [fg:red](styled one) \\[middle\\) [bg:blue;deco:bold,italic](styled two) \\\n trailing",
        ).collect::<Vec<_>>();
        insta::assert_debug_snapshot!(tokens);
    }
}
