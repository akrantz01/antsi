use super::{atoms::non_empty, specifier::style, Token, Tokens};
use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::{char, multispace1},
    combinator::{cut, map, recognize, value},
    error::{context, ContextError, ParseError},
    multi::fold_many0,
    sequence::{pair, preceded, terminated},
    AsChar, Compare, FindToken, IResult, InputIter, InputLength, InputTake, InputTakeAtPosition,
    Offset, Parser, Slice,
};
use std::ops::{RangeFrom, RangeTo};

/// Parse a piece of text into a sequence of tokens
pub(crate) fn text<I, E>(input: I) -> IResult<I, Tokens, E>
where
    I: AsRef<str>
        + Clone
        + Compare<&'static str>
        + InputIter
        + InputLength
        + InputTake
        + InputTakeAtPosition
        + Offset
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    <I as InputIter>::Item: AsChar + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    for<'a> &'a str: FindToken<<I as InputTakeAtPosition>::Item>,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "text",
        fold_many0(
            fragment,
            Tokens::default,
            |mut tokens, fragment: Fragment<I>| {
                match fragment {
                    Fragment::Literal(s) => tokens.push_str(s.as_ref()),
                    Fragment::EscapedCharacter(c) => tokens.push_char(c),
                    Fragment::EscapedWhitespace => {}
                    Fragment::Token(token) => tokens.push(token),
                }
                tokens
            },
        ),
    )(input)
}

/// A string fragment contains a fragment of text being parsed
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
enum Fragment<I> {
    /// A series of non-escaped characters
    Literal(I),
    /// A single parsed escape character
    EscapedCharacter(char),
    /// A block of escaped whitespace
    EscapedWhitespace,
    /// A nested segment of styled text
    Token(Token),
}

/// Extract a [`Fragment`] from the input
fn fragment<I, E>(input: I) -> IResult<I, Fragment<I>, E>
where
    I: AsRef<str>
        + Clone
        + Compare<&'static str>
        + InputIter
        + InputLength
        + InputTake
        + InputTakeAtPosition
        + Offset
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    <I as InputIter>::Item: AsChar + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    for<'a> &'a str: FindToken<<I as InputTakeAtPosition>::Item>,
    E: ParseError<I> + ContextError<I>,
{
    alt((
        map(styled_text, Fragment::Token),
        map(literal_string, Fragment::Literal),
        map(escaped_char, |ch| match ch {
            Some(ch) => Fragment::EscapedCharacter(ch),
            None => Fragment::EscapedWhitespace,
        }),
    ))(input)
}

/// Parse a segment of text with styling
fn styled_text<I, E>(input: I) -> IResult<I, Token, E>
where
    I: AsRef<str>
        + Clone
        + Compare<&'static str>
        + InputIter
        + InputLength
        + InputTake
        + InputTakeAtPosition
        + Offset
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    <I as InputIter>::Item: AsChar + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    for<'a> &'a str: FindToken<<I as InputTakeAtPosition>::Item>,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "styled text",
        map(pair(style, content), |(style, tokens)| Token::Styled {
            content: tokens.into(),
            style,
        }),
    )(input)
}

/// Parse the content for a piece of styled text
fn content<I, E>(input: I) -> IResult<I, Tokens, E>
where
    I: AsRef<str>
        + Clone
        + Compare<&'static str>
        + InputIter
        + InputLength
        + InputTake
        + InputTakeAtPosition
        + Offset
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    <I as InputIter>::Item: AsChar + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    for<'a> &'a str: FindToken<<I as InputTakeAtPosition>::Item>,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "content",
        preceded(char('('), cut(terminated(text, char(')')))),
    )(input)
}

/// Parse a non-empty block of text that doesn't include any escaped characters
fn literal_string<I, E>(input: I) -> IResult<I, I, E>
where
    I: AsRef<str>
        + Clone
        + InputIter
        + InputLength
        + InputTake
        + InputTakeAtPosition
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>
        + Offset,
    for<'a> &'a str: FindToken<<I as InputTakeAtPosition>::Item>,
    E: ParseError<I>,
{
    recognize(is_not("()[]\\").and_then(non_empty))(input)
}

/// Parse an escaped character: (, ), \[, \]
///
/// Characters are escaped by repeating them twice in a sequence. For example, escaping `(`
/// is done using `((`.
fn escaped_char<I, E>(input: I) -> IResult<I, Option<char>, E>
where
    I: Clone + InputIter + InputLength + InputTake + InputTakeAtPosition + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "escape sequence",
        preceded(
            char('\\'),
            cut(alt((
                value(Some('('), char('(')),
                value(Some(')'), char(')')),
                value(Some('['), char('[')),
                value(Some(']'), char(']')),
                value(Some('\\'), char('\\')),
                value(None, multispace1),
            ))),
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    mod escaped_char {
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(escaped_char -> Option<char>);

        simple_tests! {
            for escaped_char;
            open_parenthesis: "\\(" => Some('('),
            closed_parenthesis: "\\)" => Some(')'),
            open_square_bracket: "\\[" => Some('['),
            closed_square_bracket: "\\]" => Some(']'),

            whitespace_single_space: "\\ " => None,
            whitespace_multiple_spaces: "\\   " => None,
            whitespace_single_newline: "\\\n" => None,
            whitespace_multiple_newlines: "\\\n\n\n" => None,
            whitespace_single_tab: "\\\t" => None,
            whitespace_multiple_tabs: "\\\t\t\t" => None,

            whitespace_mixed_spaces: "\\ \n\t" => None,
        }

        #[test]
        fn invalid() {
            assert_eq!(
                escaped_char("\\a"),
                Err(nom::Err::Failure(error_position!(
                    "a",
                    ErrorKind::MultiSpace
                )))
            )
        }

        #[test]
        fn stops_at_non_whitespace_characters() {
            assert_eq!(escaped_char("\\ a "), Ok(("a ", None)));
            assert_eq!(escaped_char("\\ 1 "), Ok(("1 ", None)));
            assert_eq!(escaped_char("\\ ! "), Ok(("! ", None)));
        }
    }

    mod literal_string {
        make_error_concrete!(literal_string -> &str);

        simple_tests! {
            for literal_string;
            consumes_everything: "abcdef123456!@#$%^" => "abcdef123456!@#$%^",
        }

        #[test]
        fn consumes_until_open_parenthesis() {
            assert_eq!(literal_string("abcdef("), Ok(("(", "abcdef")));
        }

        #[test]
        fn consumes_until_close_parenthesis() {
            assert_eq!(literal_string("abcdef)"), Ok((")", "abcdef")));
        }

        #[test]
        fn consumes_until_open_square_bracket() {
            assert_eq!(literal_string("abcdef["), Ok(("[", "abcdef")));
        }

        #[test]
        fn consumes_until_close_square_bracket() {
            assert_eq!(literal_string("abcdef]"), Ok(("]", "abcdef")));
        }

        #[test]
        fn consumes_until_backslash() {
            assert_eq!(literal_string("abcdef\\"), Ok(("\\", "abcdef")));
        }
    }

    mod content {
        use crate::tokens::{Token, Tokens};
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(content -> Tokens);

        simple_tests! {
            for content;
            string_literal: "(literal abc 1 2 3!)" => vec![Token::Content(String::from("literal abc 1 2 3!"))],

            escaped_characters_parenthesis: "(\\(\\))" => vec![Token::Content(String::from("()"))],
            escaped_characters_square_brackets: "(\\[\\])" => vec![Token::Content(String::from("[]"))],
            escaped_characters_whitespace: "(\\  )" => vec![],

            nested_token_with_foreground: "([fg:red](inner))" => vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(fg: Red;) }],
            nested_token_with_background: "([bg:blue](inner))" => vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(bg: Blue;) }],
            nested_token_with_decoration: "([deco:dim](inner))" => vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(deco: Dim;) }],
            nested_token_with_multiple: "([deco:dim;fg:red;bg:blue](inner))" => vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(fg: Red; bg: Blue; deco: Dim;) }],

            nested_token_with_leading_content: "(leading [fg:red](content))" => vec![Token::Content(String::from("leading ")), Token::Styled { content: vec![Token::Content(String::from("content"))], style: style!(fg: Red;) }],
            nested_token_with_trailing_content: "([fg:red](content) trailing)" => vec![Token::Styled { content: vec![Token::Content(String::from("content"))], style: style!(fg: Red;) }, Token::Content(String::from(" trailing"))],
            nested_token_with_leading_and_trailing_content: "(leading [fg:red](content) trailing)" => vec![
                Token::Content(String::from("leading ")),
                Token::Styled { content: vec![Token::Content(String::from("content"))], style: style!(fg: Red;) },
                Token::Content(String::from(" trailing")),
            ],

            kitchen_sink: "(leading [fg:red](one [bg:blue](two [deco:dim](three) two) one) trailing)" => vec![
                Token::Content(String::from("leading ")),
                Token::Styled {
                    content: vec![
                        Token::Content(String::from("one ")),
                        Token::Styled {
                            content: vec![
                                Token::Content(String::from("two ")),
                                Token::Styled {
                                    content: vec![Token::Content(String::from("three"))],
                                    style: style!(deco: Dim;),
                                },
                                Token::Content(String::from(" two")),
                            ],
                            style: style!(bg: Blue;),
                        },
                        Token::Content(String::from(" one"))
                    ],
                    style: style!(fg: Red;),
                },
                Token::Content(String::from(" trailing")),
            ],
        }

        #[test]
        fn missing_closing_parenthesis() {
            assert_eq!(
                content("(test"),
                Err(nom::Err::Failure(error_position!("", ErrorKind::Char)))
            )
        }

        #[test]
        fn unescaped_open_square_bracket() {
            assert_eq!(
                content("([)"),
                Err(nom::Err::Failure(error_position!(")", ErrorKind::Tag)))
            )
        }

        #[test]
        fn unescaped_close_square_bracket() {
            assert_eq!(
                content("(])"),
                Err(nom::Err::Failure(error_position!("])", ErrorKind::Char)))
            )
        }

        #[test]
        fn unescaped_open_parenthesis() {
            assert_eq!(
                content("(()"),
                Err(nom::Err::Failure(error_position!("()", ErrorKind::Char)))
            )
        }

        #[test]
        fn unescaped_close_parenthesis() {
            assert_eq!(content("())"), Ok((")", Tokens::from(vec![]))))
        }
    }

    mod styled_text {
        use crate::tokens::Token;
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(styled_text -> Token);

        simple_tests! {
            for styled_text;
            foreground_no_content: "[fg:red]()" => Token::Styled { content: vec![], style: style!(fg: Red;) },
            background_no_content: "[bg:blue]()" => Token::Styled { content: vec![], style: style!(bg: Blue;) },
            decoration_single_style_no_content: "[deco:dim]()" => Token::Styled { content: vec![], style: style!(deco: Dim;) },
            decoration_mutliple_styles_no_content: "[deco:dim,italic]()" => Token::Styled { content: vec![], style: style!(deco: Dim, Italic;) },

            lowercase_alphabetic_content: "[fg:red](hello)" => Token::Styled { content: vec![Token::Content(String::from("hello"))], style: style!(fg: Red;) },
            uppercase_alphabetic_content: "[fg:red](HELLO)" => Token::Styled { content: vec![Token::Content(String::from("HELLO"))], style: style!(fg: Red;) },
            mixed_alphabetic_content: "[fg:red](hElLo)" => Token::Styled { content: vec![Token::Content(String::from("hElLo"))], style: style!(fg: Red;), },
            numeric_content: "[fg:red](12345)" => Token::Styled { content: vec![Token::Content(String::from("12345"))], style: style!(fg: Red;) },
            special_character_content: "[fg:red](!@#$%^)" => Token::Styled { content: vec![Token::Content(String::from("!@#$%^"))], style: style!(fg: Red;) },
            escaped_character_content: "[fg:red](\\(\\[\\]\\))" => Token::Styled { content: vec![Token::Content(String::from("([])"))], style: style!(fg: Red;) },

            nested_token: "[fg:red]([bg:blue](inner))" => Token::Styled { content: vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(bg: Blue;) }], style: style!(fg: Red;) },
            nested_token_with_leading_content: "[fg:red](leading [bg:blue](inner))" => Token::Styled {
                content: vec![
                    Token::Content(String::from("leading ")),
                    Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(bg: Blue;) },
                ],
                style: style!(fg: Red;)
            },
            nested_token_with_trailing_content: "[fg:red]([bg:blue](inner) trailing)" => Token::Styled {
                content: vec![
                    Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(bg: Blue;) },
                    Token::Content(String::from(" trailing")),
                ],
                style: style!(fg: Red;)
            },
            nested_token_with_leading_and_trailing_content: "[fg:red](leading [bg:blue](inner) trailing)" => Token::Styled {
                content: vec![
                    Token::Content(String::from("leading ")),
                    Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(bg: Blue;) },
                    Token::Content(String::from(" trailing")),
                ],
                style: style!(fg: Red;)
            },
        }

        #[test]
        fn empty() {
            assert_eq!(
                styled_text(""),
                Err(nom::Err::Error(error_position!("", ErrorKind::Char)))
            )
        }

        #[test]
        fn empty_specifier() {
            assert_eq!(
                styled_text("[]()"),
                Err(nom::Err::Failure(error_position!("]()", ErrorKind::Tag)))
            )
        }

        #[test]
        fn unclosed_specifier() {
            assert_eq!(
                styled_text("[fg:red"),
                Err(nom::Err::Failure(error_position!("", ErrorKind::Char)))
            )
        }

        #[test]
        fn unclosed_content() {
            assert_eq!(
                styled_text("[fg:red](test"),
                Err(nom::Err::Failure(error_position!("", ErrorKind::Char)))
            )
        }

        #[test]
        fn unescaped_open_square_bracket() {
            assert_eq!(
                styled_text("[fg:red]([)"),
                Err(nom::Err::Failure(error_position!(")", ErrorKind::Tag)))
            )
        }

        #[test]
        fn unescaped_close_square_bracket() {
            assert_eq!(
                styled_text("[fg:red](])"),
                Err(nom::Err::Failure(error_position!("])", ErrorKind::Char)))
            )
        }

        #[test]
        fn unescaped_open_parenthesis() {
            assert_eq!(
                styled_text("[fg:red](()"),
                Err(nom::Err::Failure(error_position!("()", ErrorKind::Char)))
            )
        }

        #[test]
        fn unescaped_close_parenthesis() {
            assert_eq!(
                styled_text("[fg:red]())"),
                Ok((
                    ")",
                    Token::Styled {
                        content: vec![],
                        style: style!(fg: Red;)
                    }
                ))
            )
        }
    }

    mod fragment {
        use crate::tokens::{content::Fragment, Token};
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(fragment -> Fragment<&str>);

        simple_tests! {
            for fragment;

            literal_string: "abcdefABCDEF123456!@#$%^" => Fragment::Literal("abcdefABCDEF123456!@#$%^"),

            escaped_open_parenthesis: "\\(" => Fragment::EscapedCharacter('('),
            escaped_close_parenthesis: "\\)" => Fragment::EscapedCharacter(')'),
            escaped_open_square_bracket: "\\[" => Fragment::EscapedCharacter('['),
            escaped_close_square_bracket: "\\]" => Fragment::EscapedCharacter(']'),

            escaped_whitespace_single_space: "\\ " => Fragment::EscapedWhitespace,
            escaped_whitespace_multiple_spaces: "\\   " => Fragment::EscapedWhitespace,
            escaped_whitespace_single_tab: "\\\t" => Fragment::EscapedWhitespace,
            escaped_whitespace_multiple_tabs: "\\\t\t\t" => Fragment::EscapedWhitespace,
            escaped_whitespace_single_newline: "\\\n" => Fragment::EscapedWhitespace,
            escaped_whitespace_multiple_newlines: "\\\n\n\n" => Fragment::EscapedWhitespace,
            escaped_whitespace_mixed: "\\ \n\t" => Fragment::EscapedWhitespace,

            token_foreground_no_content: "[fg:red]()" => Fragment::Token(Token::Styled { content: vec![], style: style!(fg: Red;) }),
            token_background_no_content: "[bg:blue]()" => Fragment::Token(Token::Styled { content: vec![], style: style!(bg: Blue;) }),
            token_decoration_single_style_no_content: "[deco:dim]()" => Fragment::Token(Token::Styled { content: vec![], style: style!(deco: Dim;) }),
            token_decoration_mutliple_styles_no_content: "[deco:dim,italic]()" => Fragment::Token(Token::Styled { content: vec![], style: style!(deco: Dim, Italic;) }),

            token_lowercase_alphabetic_content: "[fg:red](hello)" => Fragment::Token(Token::Styled { content: vec![Token::Content(String::from("hello"))], style: style!(fg: Red;) }),
            token_uppercase_alphabetic_content: "[fg:red](HELLO)" => Fragment::Token(Token::Styled { content: vec![Token::Content(String::from("HELLO"))], style: style!(fg: Red;) }),
            token_mixed_alphabetic_content: "[fg:red](hElLo)" => Fragment::Token(Token::Styled { content: vec![Token::Content(String::from("hElLo"))], style: style!(fg: Red;), }),
            token_numeric_content: "[fg:red](12345)" => Fragment::Token(Token::Styled { content: vec![Token::Content(String::from("12345"))], style: style!(fg: Red;) }),
            token_special_character_content: "[fg:red](!@#$%^)" => Fragment::Token(Token::Styled { content: vec![Token::Content(String::from("!@#$%^"))], style: style!(fg: Red;) }),
            token_escaped_character_content: "[fg:red](\\(\\[\\]\\))" => Fragment::Token(Token::Styled { content: vec![Token::Content(String::from("([])"))], style: style!(fg: Red;) }),

            nested_token: "[fg:red]([bg:blue](inner))" => Fragment::Token(Token::Styled { content: vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(bg: Blue;) }], style: style!(fg: Red;) }),
            nested_token_with_leading_content: "[fg:red](leading [bg:blue](inner))" => Fragment::Token(Token::Styled {
                content: vec![
                    Token::Content(String::from("leading ")),
                    Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(bg: Blue;) },
                ],
                style: style!(fg: Red;)
            }),
            nested_token_with_trailing_content: "[fg:red]([bg:blue](inner) trailing)" => Fragment::Token(Token::Styled {
                content: vec![
                    Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(bg: Blue;) },
                    Token::Content(String::from(" trailing")),
                ],
                style: style!(fg: Red;)
            }),
            nested_token_with_leading_and_trailing_content: "[fg:red](leading [bg:blue](inner) trailing)" => Fragment::Token(Token::Styled {
                content: vec![
                    Token::Content(String::from("leading ")),
                    Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(bg: Blue;) },
                    Token::Content(String::from(" trailing")),
                ],
                style: style!(fg: Red;)
            }),
        }

        #[test]
        fn stops_after_unescaped_open_parenthesis() {
            assert_eq!(
                fragment("before ( after"),
                Ok(("( after", Fragment::Literal("before ")))
            )
        }

        #[test]
        fn stops_after_unescaped_close_parenthesis() {
            assert_eq!(
                fragment("before ) after"),
                Ok((") after", Fragment::Literal("before ")))
            )
        }

        #[test]
        fn stops_after_unescaped_open_square_bracket() {
            assert_eq!(
                fragment("before [ after"),
                Ok(("[ after", Fragment::Literal("before ")))
            )
        }

        #[test]
        fn stops_after_unescaped_close_square_bracket() {
            assert_eq!(
                fragment("before ] after"),
                Ok(("] after", Fragment::Literal("before ")))
            )
        }

        #[test]
        fn stops_after_backslash() {
            assert_eq!(
                fragment("before \\ after"),
                Ok(("\\ after", Fragment::Literal("before ")))
            )
        }

        #[test]
        fn invalid_escape_character() {
            assert_eq!(
                fragment("\\a"),
                Err(nom::Err::Failure(error_position!(
                    "a",
                    ErrorKind::MultiSpace
                )))
            )
        }
    }

    mod text {
        use crate::tokens::{Token, Tokens};
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(text -> Tokens);

        simple_tests! {
            for text;

            empty: "" => vec![],

            lowercase_alphabetic: "abcdef" => vec![Token::Content(String::from("abcdef"))],
            uppercase_alphabetic: "ABCDEF" => vec![Token::Content(String::from("ABCDEF"))],
            mixed_case_alphabetic: "aBcDeF" => vec![Token::Content(String::from("aBcDeF"))],
            numeric: "123456" => vec![Token::Content(String::from("123456"))],
            lowercase_alphanumeric: "abc123" => vec![Token::Content(String::from("abc123"))],
            uppercase_alphanumeric: "ABC123" => vec![Token::Content(String::from("ABC123"))],
            mixed_case_alphanumeric: "AbCd1234" => vec![Token::Content(String::from("AbCd1234"))],
            special_characters: "!@#$%^" => vec![Token::Content(String::from("!@#$%^"))],
            mixed_characters: "ABCdef123!@#" => vec![Token::Content(String::from("ABCdef123!@#"))],

            escaped_characters: "\\(\\)\\[\\]" => vec![Token::Content(String::from("()[]"))],
            escaped_whitespace: "\\ \n\t" => vec![],

            mixed_characters_and_escape_characters: "abc\\(DEF\\)12\\   34\\[!@#$\\]" => vec![Token::Content(String::from("abc(DEF)1234[!@#$]"))],

            empty_token: "[fg:red]()" => vec![Token::Styled { content: vec![], style: style!(fg: Red;) }],
            token_with_foreground: "[fg:red](inner)" => vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(fg: Red;) }],
            token_with_background: "[bg:blue](inner)" => vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(bg: Blue;) }],
            token_with_single_decoration: "[deco:dim](inner)" => vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(deco: Dim;) }],
            token_with_multiple_decorations: "[deco:dim,italic](inner)" => vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(deco: Dim, Italic;) }],
            token_with_multiple_styles: "[deco:dim,italic;fg:red;bg:blue](inner)" => vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(fg: Red; bg: Blue; deco: Dim, Italic;) }],

            token_with_leading_content: "leading [fg:red](content)" => vec![
                Token::Content(String::from("leading ")),
                Token::Styled { content: vec![Token::Content(String::from("content"))], style: style!(fg: Red;) },
            ],
            token_with_trailing_content: "[fg:red](content) trailing" => vec![
                Token::Styled { content: vec![Token::Content(String::from("content"))], style: style!(fg: Red;) },
                Token::Content(String::from(" trailing")),
            ],
            token_with_leading_and_trailing_content: "leading [fg:red](content) trailing" => vec![
                Token::Content(String::from("leading ")),
                Token::Styled { content: vec![Token::Content(String::from("content"))], style: style!(fg: Red;) },
                Token::Content(String::from(" trailing")),
            ],

            nested_token: "[fg:red]([bg:blue](inner))" => vec![Token::Styled { content: vec![Token::Styled { content: vec![Token::Content(String::from("inner"))], style: style!(bg: Blue;) }], style: style!(fg: Red;) }],

            kitchen_sink: "leading [fg:red](one [bg:blue](two [deco:dim](three) two) one) trailing" => vec![
                Token::Content(String::from("leading ")),
                Token::Styled {
                    content: vec![
                        Token::Content(String::from("one ")),
                        Token::Styled {
                            content: vec![
                                Token::Content(String::from("two ")),
                                Token::Styled {
                                    content: vec![Token::Content(String::from("three"))],
                                    style: style!(deco: Dim;),
                                },
                                Token::Content(String::from(" two")),
                            ],
                            style: style!(bg: Blue;),
                        },
                        Token::Content(String::from(" one"))
                    ],
                    style: style!(fg: Red;),
                },
                Token::Content(String::from(" trailing")),
            ],
        }

        #[test]
        fn unescaped_open_parenthesis_in_plaintext() {
            assert_eq!(
                text("before ( after"),
                Ok((
                    "( after",
                    Tokens::from(vec![Token::Content(String::from("before "))])
                ))
            )
        }

        #[test]
        fn unescaped_close_parenthesis_in_plaintext() {
            assert_eq!(
                text("before ) after"),
                Ok((
                    ") after",
                    Tokens::from(vec![Token::Content(String::from("before "))])
                ))
            )
        }

        #[test]
        fn unescaped_open_square_bracket_in_plaintext() {
            assert_eq!(
                text("before [ after"),
                Err(nom::Err::Failure(error_position!("after", ErrorKind::Tag)))
            )
        }

        #[test]
        fn unescaped_close_square_bracket_in_plaintext() {
            assert_eq!(
                text("before ] after"),
                Ok((
                    "] after",
                    Tokens::from(vec![Token::Content(String::from("before "))])
                ))
            )
        }

        #[test]
        fn unescaped_open_parenthesis_in_token() {
            assert_eq!(
                text("[fg:red](before ( after)"),
                Err(nom::Err::Failure(error_position!(
                    "( after)",
                    ErrorKind::Char
                )))
            )
        }

        #[test]
        fn unescaped_close_parenthesis_in_token() {
            assert_eq!(
                text("[fg:red](before ) after)"),
                Ok((
                    ")",
                    Tokens::from(vec![
                        Token::Styled {
                            content: vec![Token::Content(String::from("before "))],
                            style: style!(fg: Red;)
                        },
                        Token::Content(String::from(" after"))
                    ])
                ))
            )
        }

        #[test]
        fn unescaped_open_square_bracket_in_token() {
            assert_eq!(
                text("[fg:red](before [ after)"),
                Err(nom::Err::Failure(error_position!("after)", ErrorKind::Tag)))
            )
        }

        #[test]
        fn unescaped_close_square_bracket_in_token() {
            assert_eq!(
                text("[fg:red](before ] after)"),
                Err(nom::Err::Failure(error_position!(
                    "] after)",
                    ErrorKind::Char
                )))
            )
        }

        #[test]
        fn token_empty_specifier() {
            assert_eq!(
                text("[]()"),
                Err(nom::Err::Failure(error_position!("]()", ErrorKind::Tag)))
            )
        }

        #[test]
        fn token_unclosed_specifier() {
            assert_eq!(
                text("[fg:red"),
                Err(nom::Err::Failure(error_position!("", ErrorKind::Char)))
            )
        }

        #[test]
        fn token_unclosed_content() {
            assert_eq!(
                text("[fg:red](test"),
                Err(nom::Err::Failure(error_position!("", ErrorKind::Char)))
            )
        }

        #[test]
        fn token_bad_escape_character() {
            assert_eq!(
                text("before \\a after"),
                Err(nom::Err::Failure(error_position!(
                    "a after",
                    ErrorKind::MultiSpace
                )))
            )
        }
    }
}
