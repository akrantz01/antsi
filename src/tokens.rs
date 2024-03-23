use crate::styles::Style;
use nom::{
    combinator::{all_consuming, complete},
    error::{ContextError, ParseError},
    AsChar, Compare, FindToken, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset,
    Slice,
};
use std::ops::{RangeFrom, RangeTo};

#[cfg(test)]
#[macro_use]
mod macros;
mod atoms;
mod content;
mod specifier;

/// One or more pieces of text that either apply additional styling or inherit from the parent
/// styles.
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub(crate) enum Token {
    /// A piece of text that does not modify the styling
    Content(String),
    /// One or more pieces of text that with additional styling
    Styled {
        /// The pieces of text the style applies to
        content: Vec<Token>,
        /// The style to apply
        style: Style,
    },
}

/// A sequence of [`Token`]s
#[derive(Clone, Debug, Default)]
#[cfg_attr(test, derive(Eq, PartialEq))]
struct Tokens(Vec<Token>);

impl From<Vec<Token>> for Tokens {
    fn from(tokens: Vec<Token>) -> Self {
        Tokens(tokens)
    }
}

impl From<Tokens> for Vec<Token> {
    fn from(tokens: Tokens) -> Self {
        tokens.0
    }
}

impl Tokens {
    /// Add a new token to the end of the sequence
    fn push(&mut self, token: Token) {
        self.0.push(token)
    }

    /// Add a string to the end of the sequence
    ///
    /// If the last token in the sequence is an unstyled piece of text, it will be appended directly
    /// to the token. Otherwise, a new content token will be created.
    fn push_str(&mut self, s: &str) {
        match self.0.last_mut() {
            Some(Token::Content(content)) => content.push_str(s),
            Some(Token::Styled { .. }) | None => self.0.push(Token::Content(s.to_string())),
        }
    }

    /// Add a character to the end of the sequence
    ///
    /// If the last token in the sequence is an unstyled piece of text, it will be appended directly
    /// to the token. Otherwise, a new content token will be created.
    fn push_char(&mut self, ch: char) {
        match self.0.last_mut() {
            Some(Token::Content(content)) => content.push(ch),
            Some(Token::Styled { .. }) | None => self.0.push(Token::Content(ch.to_string())),
        }
    }
}

/// Parse a piece of text into a sequence of [`Token`]s for processing
pub(crate) fn tokenize<I, E>(input: I) -> Result<Vec<Token>, E>
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
    match all_consuming(complete(content::text))(input) {
        Ok((_leftover, tokens)) => Ok(tokens.into()),
        Err(nom::Err::Error(e) | nom::Err::Failure(e)) => Err(e),
        Err(nom::Err::Incomplete(_)) => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    mod tokens {
        use crate::tokens::{Token, Tokens};

        #[test]
        fn push_adds_token_to_end_when_no_tokens() {
            let mut tokens = Tokens::default();
            tokens.push(Token::Content(String::from("test")));

            assert_eq!(
                tokens,
                Tokens::from(vec![Token::Content(String::from("test"))])
            )
        }

        #[test]
        fn push_adds_token_to_end_when_tokens_present() {
            let mut tokens = Tokens::from(vec![Token::Content(String::from("existing"))]);
            tokens.push(Token::Content(String::from("test")));

            assert_eq!(
                tokens,
                Tokens::from(vec![
                    Token::Content(String::from("existing")),
                    Token::Content(String::from("test"))
                ])
            )
        }

        #[test]
        fn push_str_adds_new_content_token_to_end_when_no_tokens() {
            let mut tokens = Tokens::default();
            tokens.push_str("test");

            assert_eq!(
                tokens,
                Tokens::from(vec![Token::Content(String::from("test"))])
            )
        }

        #[test]
        fn push_str_adds_new_content_token_to_end_when_empty_style_token_present() {
            let mut tokens = Tokens::from(vec![Token::Styled {
                content: vec![],
                style: style!(),
            }]);
            tokens.push_str("test");

            assert_eq!(
                tokens,
                Tokens::from(vec![
                    Token::Styled {
                        content: vec![],
                        style: style!()
                    },
                    Token::Content(String::from("test"))
                ])
            )
        }

        #[test]
        fn push_str_adds_new_content_token_to_end_when_style_token_with_children_exists() {
            let mut tokens = Tokens::from(vec![Token::Styled {
                content: vec![Token::Content(String::from("existing"))],
                style: style!(),
            }]);
            tokens.push_str("test");

            assert_eq!(
                tokens,
                Tokens::from(vec![
                    Token::Styled {
                        content: vec![Token::Content(String::from("existing"))],
                        style: style!(),
                    },
                    Token::Content(String::from("test"))
                ])
            );
        }

        #[test]
        fn push_str_appends_to_last_non_nested_content_token() {
            let mut tokens = Tokens::from(vec![Token::Content(String::from("existing "))]);
            tokens.push_str("test");

            assert_eq!(
                tokens,
                Tokens::from(vec![Token::Content(String::from("existing test"))])
            );
        }

        #[test]
        fn push_char_adds_new_content_token_to_end_when_no_tokens() {
            let mut tokens = Tokens::default();
            tokens.push_char('T');

            assert_eq!(
                tokens,
                Tokens::from(vec![Token::Content(String::from("T"))])
            )
        }

        #[test]
        fn push_char_adds_new_content_token_to_end_when_empty_style_token_present() {
            let mut tokens = Tokens::from(vec![Token::Styled {
                content: vec![],
                style: style!(),
            }]);
            tokens.push_char('T');

            assert_eq!(
                tokens,
                Tokens::from(vec![
                    Token::Styled {
                        content: vec![],
                        style: style!()
                    },
                    Token::Content(String::from("T"))
                ])
            )
        }

        #[test]
        fn push_char_adds_new_content_token_to_end_when_style_token_with_children_exists() {
            let mut tokens = Tokens::from(vec![Token::Styled {
                content: vec![Token::Content(String::from("existing"))],
                style: style!(),
            }]);
            tokens.push_char('T');

            assert_eq!(
                tokens,
                Tokens::from(vec![
                    Token::Styled {
                        content: vec![Token::Content(String::from("existing"))],
                        style: style!(),
                    },
                    Token::Content(String::from("T"))
                ])
            );
        }

        #[test]
        fn push_char_appends_to_last_non_nested_content_token() {
            let mut tokens = Tokens::from(vec![Token::Content(String::from("existing "))]);
            tokens.push_char('T');

            assert_eq!(
                tokens,
                Tokens::from(vec![Token::Content(String::from("existing T"))])
            );
        }
    }

    mod tokenize {
        use crate::tokens::Token;
        use nom::{
            error::{Error, ErrorKind},
            error_position,
        };

        fn tokenize(input: &str) -> Result<Vec<Token>, Error<&str>> {
            super::super::tokenize(input)
        }

        macro_rules! simple_tests {
            (
                for $function:ident;
                $( $name:ident: $input:literal => $output:expr ),* $(,)?
            ) => {
                $(
                    #[test]
                    fn $name () {
                        assert_eq!($function($input), Ok(From::from($output)));
                    }
                )*
            };
        }

        simple_tests! {
            for tokenize;

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
                tokenize("before ( after"),
                Err(error_position!("( after", ErrorKind::Eof))
            );
        }

        #[test]
        fn unescaped_close_parenthesis_in_plaintext() {
            assert_eq!(
                tokenize("before ) after"),
                Err(error_position!(") after", ErrorKind::Eof))
            )
        }

        #[test]
        fn unescaped_open_square_bracket_in_plaintext() {
            assert_eq!(
                tokenize("before [ after"),
                Err(error_position!("after", ErrorKind::Tag))
            )
        }

        #[test]
        fn unescaped_close_square_bracket_in_plaintext() {
            assert_eq!(
                tokenize("before ] after"),
                Err(error_position!("] after", ErrorKind::Eof))
            )
        }

        #[test]
        fn unescaped_open_parenthesis_in_token() {
            assert_eq!(
                tokenize("[fg:red](before ( after)"),
                Err(error_position!("( after)", ErrorKind::Char))
            )
        }

        #[test]
        fn unescaped_close_parenthesis_in_token() {
            assert_eq!(
                tokenize("[fg:red](before ) after)"),
                Err(error_position!(")", ErrorKind::Eof))
            )
        }

        #[test]
        fn unescaped_open_square_bracket_in_token() {
            assert_eq!(
                tokenize("[fg:red](before [ after)"),
                Err(error_position!("after)", ErrorKind::Tag))
            )
        }

        #[test]
        fn unescaped_close_square_bracket_in_token() {
            assert_eq!(
                tokenize("[fg:red](before ] after)"),
                Err(error_position!("] after)", ErrorKind::Char))
            )
        }

        #[test]
        fn token_empty_specifier() {
            assert_eq!(
                tokenize("[]()"),
                Err(error_position!("]()", ErrorKind::Tag))
            )
        }

        #[test]
        fn token_unclosed_specifier() {
            assert_eq!(
                tokenize("[fg:red"),
                Err(error_position!("", ErrorKind::Char))
            )
        }

        #[test]
        fn token_unclosed_content() {
            assert_eq!(
                tokenize("[fg:red](test"),
                Err(error_position!("", ErrorKind::Char))
            )
        }

        #[test]
        fn token_bad_escape_character() {
            assert_eq!(
                tokenize("before \\a after"),
                Err(error_position!("a after", ErrorKind::MultiSpace))
            )
        }
    }
}
