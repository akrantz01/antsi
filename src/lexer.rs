use crate::styles::{Color, Decoration};
use logos::Logos;
use std::{
    fmt::{Display, Formatter},
    ops::Range,
    str::FromStr,
};

pub(crate) struct Lexer<'source>(logos::Lexer<'source, SyntaxKind>);

impl<'source> Lexer<'source> {
    pub fn new(input: &'source str) -> Self {
        Self(SyntaxKind::lexer(input))
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Result<Token<'source>, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.0.next()?;

        Some(kind.map(|kind| Token {
            kind,
            text: self.0.slice(),
            span: self.0.span(),
        }))
    }
}

#[derive(Clone, Copy, Debug, Logos, PartialEq)]
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
        |lex| Color::from_str(lex.slice()).expect("valid color"),
        priority = 10,
        ignore(ascii_case)
    )]
    #[token("default", |_| Color::Default, ignore(ascii_case))]
    Color(Color),

    #[regex(
        r#"(bold|dim|faint|italic|underline|(fast|slow)-blink|invert|reverse|hide|conceal|strike(-)?through)"#,
        |lex| Decoration::from_str(lex.slice()).expect("valid decoration"),
        priority = 10,
        ignore(ascii_case)
    )]
    Decoration(Decoration),

    #[regex(r#"\\[\\\[\]()]"#, |lex| lex.slice().chars().nth(1).unwrap())]
    EscapeCharacter(char),

    #[regex(r#"\\[ \r\n\t]+"#)]
    EscapeWhitespace,

    // as a temporary fix until https://github.com/maciejhirsz/logos/issues/265 is resolved, the
    // tokens `:` `;` and `,` are considered stop characters for words
    #[regex(r#"[^\\\[\]():;,]+"#, priority = 2)]
    Text,
}

impl Display for SyntaxKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
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
            Self::Color(_) => "color",
            Self::Decoration(_) => "decoration",
            Self::EscapeCharacter(_) => "escape character",
            Self::EscapeWhitespace => "escape whitespace",
            Self::Text => "text",
        })
    }
}

#[derive(Debug)]
pub(crate) struct Token<'source> {
    pub kind: SyntaxKind,
    pub text: &'source str,
    pub span: Range<usize>,
}

#[cfg(test)]
mod tests {
    use super::{Lexer, SyntaxKind};
    use crate::styles::{Color, Decoration};

    fn check(input: &str, kind: SyntaxKind) {
        let mut lexer = Lexer::new(input);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token.kind, kind);
        assert_eq!(token.text, input);
        assert_ne!(token.span.len(), 0);
    }

    fn check_many(input: &str, tokens: Vec<(SyntaxKind, &str)>) {
        let actual = Lexer::new(input)
            .map(|token| {
                let token = token.unwrap();
                (token.kind, token.text)
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, tokens);
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
        check("black", SyntaxKind::Color(Color::Black));
    }

    #[test]
    fn color_red() {
        check("red", SyntaxKind::Color(Color::Red));
    }

    #[test]
    fn color_green() {
        check("green", SyntaxKind::Color(Color::Green));
    }

    #[test]
    fn color_yellow() {
        check("yellow", SyntaxKind::Color(Color::Yellow));
    }

    #[test]
    fn color_blue() {
        check("blue", SyntaxKind::Color(Color::Blue));
    }

    #[test]
    fn color_magenta() {
        check("magenta", SyntaxKind::Color(Color::Magenta));
    }

    #[test]
    fn color_cyan() {
        check("cyan", SyntaxKind::Color(Color::Cyan));
    }

    #[test]
    fn color_white() {
        check("white", SyntaxKind::Color(Color::White));
    }

    #[test]
    fn color_default() {
        check("default", SyntaxKind::Color(Color::Default));
    }

    #[test]
    fn color_bright_black() {
        check("bright-black", SyntaxKind::Color(Color::BrightBlack));
    }

    #[test]
    fn color_bright_red() {
        check("bright-red", SyntaxKind::Color(Color::BrightRed));
    }

    #[test]
    fn color_bright_green() {
        check("bright-green", SyntaxKind::Color(Color::BrightGreen));
    }

    #[test]
    fn color_bright_yellow() {
        check("bright-yellow", SyntaxKind::Color(Color::BrightYellow));
    }

    #[test]
    fn color_bright_blue() {
        check("bright-blue", SyntaxKind::Color(Color::BrightBlue));
    }

    #[test]
    fn color_bright_magenta() {
        check("bright-magenta", SyntaxKind::Color(Color::BrightMagenta));
    }

    #[test]
    fn color_bright_cyan() {
        check("bright-cyan", SyntaxKind::Color(Color::BrightCyan));
    }

    #[test]
    fn color_bright_white() {
        check("bright-white", SyntaxKind::Color(Color::BrightWhite));
    }

    #[test]
    fn decoration_bold() {
        check("bold", SyntaxKind::Decoration(Decoration::Bold));
    }

    #[test]
    fn decoration_dim() {
        check("dim", SyntaxKind::Decoration(Decoration::Dim));
    }

    #[test]
    fn decoration_faint() {
        check("faint", SyntaxKind::Decoration(Decoration::Dim));
    }

    #[test]
    fn decoration_italic() {
        check("italic", SyntaxKind::Decoration(Decoration::Italic));
    }

    #[test]
    fn decoration_underline() {
        check("underline", SyntaxKind::Decoration(Decoration::Underline));
    }

    #[test]
    fn decoration_fast_blink() {
        check("fast-blink", SyntaxKind::Decoration(Decoration::FastBlink));
    }

    #[test]
    fn decoration_slow_blink() {
        check("slow-blink", SyntaxKind::Decoration(Decoration::SlowBlink));
    }

    #[test]
    fn decoration_invert() {
        check("invert", SyntaxKind::Decoration(Decoration::Invert));
    }

    #[test]
    fn decoration_reverse() {
        check("reverse", SyntaxKind::Decoration(Decoration::Invert));
    }

    #[test]
    fn decoration_hide() {
        check("hide", SyntaxKind::Decoration(Decoration::Hide));
    }

    #[test]
    fn decoration_conceal() {
        check("conceal", SyntaxKind::Decoration(Decoration::Hide));
    }

    #[test]
    fn decoration_strikethrough() {
        check(
            "strikethrough",
            SyntaxKind::Decoration(Decoration::StrikeThrough),
        );
    }

    #[test]
    fn decoration_strike_through() {
        check(
            "strike-through",
            SyntaxKind::Decoration(Decoration::StrikeThrough),
        );
    }

    #[test]
    fn escape_character_backslash() {
        check("\\\\", SyntaxKind::EscapeCharacter('\\'));
    }

    #[test]
    fn escape_character_open_square_bracket() {
        check("\\[", SyntaxKind::EscapeCharacter('['));
    }

    #[test]
    fn escape_character_close_square_bracket() {
        check("\\]", SyntaxKind::EscapeCharacter(']'));
    }

    #[test]
    fn escape_character_open_parenthesis() {
        check("\\(", SyntaxKind::EscapeCharacter('('));
    }

    #[test]
    fn escape_character_close_parenthesis() {
        check("\\)", SyntaxKind::EscapeCharacter(')'));
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
        check_many(
            "fg:blue",
            vec![
                (SyntaxKind::ForegroundSpecifier, "fg"),
                (SyntaxKind::Colon, ":"),
                (SyntaxKind::Color(Color::Blue), "blue"),
            ],
        );
    }

    #[test]
    fn background_style_specifier() {
        check_many(
            "bg:magenta",
            vec![
                (SyntaxKind::BackgroundSpecifier, "bg"),
                (SyntaxKind::Colon, ":"),
                (SyntaxKind::Color(Color::Magenta), "magenta"),
            ],
        );
    }

    #[test]
    fn single_decoration_style_specifier() {
        check_many(
            "deco:bold",
            vec![
                (SyntaxKind::DecorationSpecifier, "deco"),
                (SyntaxKind::Colon, ":"),
                (SyntaxKind::Decoration(Decoration::Bold), "bold"),
            ],
        );
    }

    #[test]
    fn multiple_decoration_style_specifiers() {
        check_many(
            "deco:bold,italic",
            vec![
                (SyntaxKind::DecorationSpecifier, "deco"),
                (SyntaxKind::Colon, ":"),
                (SyntaxKind::Decoration(Decoration::Bold), "bold"),
                (SyntaxKind::Comma, ","),
                (SyntaxKind::Decoration(Decoration::Italic), "italic"),
            ],
        );
    }

    #[test]
    fn many_tokens() {
        check_many(
            "leading [fg:red](styled one) \\[middle\\) [bg:blue;deco:bold,italic](styled two) \\\n trailing",
            vec![
                (SyntaxKind::Text, "leading "),
                (SyntaxKind::SquareBracketOpen, "["),
                (SyntaxKind::ForegroundSpecifier, "fg"),
                (SyntaxKind::Colon, ":"),
                (SyntaxKind::Color(Color::Red), "red"),
                (SyntaxKind::SquareBracketClose, "]"),
                (SyntaxKind::ParenthesisOpen, "("),
                (SyntaxKind::Text, "styled one"),
                (SyntaxKind::ParenthesisClose, ")"),
                (SyntaxKind::Text, " "),
                (SyntaxKind::EscapeCharacter('['), "\\["),
                (SyntaxKind::Text, "middle"),
                (SyntaxKind::EscapeCharacter(')'), "\\)"),
                (SyntaxKind::Text, " "),
                (SyntaxKind::SquareBracketOpen, "["),
                (SyntaxKind::BackgroundSpecifier, "bg"),
                (SyntaxKind::Colon, ":"),
                (SyntaxKind::Color(Color::Blue), "blue"),
                (SyntaxKind::Semicolon, ";"),
                (SyntaxKind::DecorationSpecifier, "deco"),
                (SyntaxKind::Colon, ":"),
                (SyntaxKind::Decoration(Decoration::Bold), "bold"),
                (SyntaxKind::Comma, ","),
                (SyntaxKind::Decoration(Decoration::Italic), "italic"),
                (SyntaxKind::SquareBracketClose, "]"),
                (SyntaxKind::ParenthesisOpen, "("),
                (SyntaxKind::Text, "styled two"),
                (SyntaxKind::ParenthesisClose, ")"),
                (SyntaxKind::Text, " "),
                (SyntaxKind::EscapeWhitespace, "\\\n "),
                (SyntaxKind::Text, "trailing"),
            ],
        )
    }
}
