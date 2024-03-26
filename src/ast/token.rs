use super::Style;

/// One or more pieces of text that either apply additional styling or inherit from the parent
/// styles.
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum Token {
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
pub struct Tokens(Vec<Token>);

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
    pub fn push(&mut self, token: Token) {
        self.0.push(token)
    }

    /// Add a string to the end of the sequence
    ///
    /// If the last token in the sequence is an unstyled piece of text, it will be appended directly
    /// to the token. Otherwise, a new content token will be created.
    pub fn push_str(&mut self, s: &str) {
        match self.0.last_mut() {
            Some(Token::Content(content)) => content.push_str(s),
            Some(Token::Styled { .. }) | None => self.0.push(Token::Content(s.to_string())),
        }
    }

    /// Add a character to the end of the sequence
    ///
    /// If the last token in the sequence is an unstyled piece of text, it will be appended directly
    /// to the token. Otherwise, a new content token will be created.
    pub fn push_char(&mut self, ch: char) {
        match self.0.last_mut() {
            Some(Token::Content(content)) => content.push(ch),
            Some(Token::Styled { .. }) | None => self.0.push(Token::Content(ch.to_string())),
        }
    }

    /// Add all the tokens from another sequence onto the current one
    pub fn extend<T>(&mut self, other: T)
    where
        T: Into<Tokens>,
    {
        let other = other.into();
        if other.0.is_empty() {
            return;
        } else if self.0.is_empty() {
            *self = other;
            return;
        }

        let mut tokens = other.0.into_iter().peekable();
        if let Some(Token::Content(last)) = self.0.last_mut() {
            while let Some(Token::Content(content)) = tokens.peek() {
                last.push_str(content);
                tokens.next();
            }
        }

        self.0.extend(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, Tokens};

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

    #[test]
    fn extend_both_empty() {
        let mut tokens = Tokens::default();
        tokens.extend(Tokens::default());
        assert_eq!(tokens, Tokens::from(vec![]));
    }

    #[test]
    fn extend_empty_with_content_token() {
        let mut tokens = Tokens::default();
        tokens.extend(vec![Token::Content(String::from("testing"))]);
        assert_eq!(
            tokens,
            Tokens::from(vec![Token::Content(String::from("testing"))])
        );
    }

    #[test]
    fn extend_empty_with_styled_token() {
        let mut tokens = Tokens::default();
        tokens.extend(vec![Token::Styled {
            style: style!(),
            content: vec![Token::Content(String::from("testing"))],
        }]);
        assert_eq!(
            tokens,
            Tokens::from(vec![Token::Styled {
                style: style!(),
                content: vec![Token::Content(String::from("testing"))]
            }])
        );
    }

    #[test]
    fn extend_single_content_token_with_empty() {
        let mut tokens = Tokens::from(vec![Token::Content(String::from("existing"))]);
        tokens.extend(vec![]);
        assert_eq!(
            tokens,
            Tokens::from(vec![Token::Content(String::from("existing"))])
        );
    }

    #[test]
    fn extend_single_content_token_with_content_token() {
        let mut tokens = Tokens::from(vec![Token::Content(String::from("existing"))]);
        tokens.extend(vec![Token::Content(String::from("testing"))]);
        assert_eq!(
            tokens,
            Tokens::from(vec![Token::Content(String::from("existingtesting"))])
        );
    }

    #[test]
    fn extend_single_content_token_with_styled_token() {
        let mut tokens = Tokens::from(vec![Token::Content(String::from("existing"))]);
        tokens.extend(vec![Token::Styled {
            style: style!(),
            content: vec![Token::Content(String::from("testing"))],
        }]);
        assert_eq!(
            tokens,
            Tokens::from(vec![
                Token::Content(String::from("existing")),
                Token::Styled {
                    style: style!(),
                    content: vec![Token::Content(String::from("testing"))],
                }
            ])
        );
    }

    #[test]
    fn extend_single_styled_token_with_empty() {
        let mut tokens = Tokens::from(vec![Token::Styled {
            style: style!(),
            content: vec![Token::Content(String::from("testing"))],
        }]);
        tokens.extend(vec![]);
        assert_eq!(
            tokens,
            Tokens::from(vec![Token::Styled {
                style: style!(),
                content: vec![Token::Content(String::from("testing"))],
            }])
        );
    }

    #[test]
    fn extend_single_styled_token_with_content_token() {
        let mut tokens = Tokens::from(vec![Token::Styled {
            style: style!(),
            content: vec![Token::Content(String::from("testing"))],
        }]);
        tokens.extend(vec![Token::Content(String::from("testing"))]);
        assert_eq!(
            tokens,
            Tokens::from(vec![
                Token::Styled {
                    style: style!(),
                    content: vec![Token::Content(String::from("testing"))],
                },
                Token::Content(String::from("testing"))
            ])
        );
    }

    #[test]
    fn extend_single_styled_token_with_styled_token() {
        let mut tokens = Tokens::from(vec![Token::Styled {
            style: style!(),
            content: vec![Token::Content(String::from("testing"))],
        }]);
        tokens.extend(vec![Token::Styled {
            style: style!(),
            content: vec![Token::Content(String::from("testing"))],
        }]);
        assert_eq!(
            tokens,
            Tokens::from(vec![
                Token::Styled {
                    style: style!(),
                    content: vec![Token::Content(String::from("testing"))],
                },
                Token::Styled {
                    style: style!(),
                    content: vec![Token::Content(String::from("testing"))],
                }
            ])
        );
    }

    #[test]
    fn extend_styled_and_content_token_with_styled_token() {
        let mut tokens = Tokens::from(vec![
            Token::Styled {
                style: style!(),
                content: vec![Token::Content(String::from("existing styled"))],
            },
            Token::Content(String::from("existing content")),
        ]);
        tokens.extend(vec![Token::Styled {
            style: style!(),
            content: vec![Token::Content(String::from("testing"))],
        }]);
        assert_eq!(
            tokens,
            Tokens::from(vec![
                Token::Styled {
                    style: style!(),
                    content: vec![Token::Content(String::from("existing styled"))],
                },
                Token::Content(String::from("existing content")),
                Token::Styled {
                    style: style!(),
                    content: vec![Token::Content(String::from("testing"))],
                }
            ])
        );
    }

    #[test]
    fn extend_content_and_styled_token_with_styled_token() {
        let mut tokens = Tokens::from(vec![
            Token::Content(String::from("existing content")),
            Token::Styled {
                style: style!(),
                content: vec![Token::Content(String::from("existing styled"))],
            },
        ]);
        tokens.extend(vec![Token::Styled {
            style: style!(),
            content: vec![Token::Content(String::from("testing"))],
        }]);
        assert_eq!(
            tokens,
            Tokens::from(vec![
                Token::Content(String::from("existing content")),
                Token::Styled {
                    style: style!(),
                    content: vec![Token::Content(String::from("existing styled"))],
                },
                Token::Styled {
                    style: style!(),
                    content: vec![Token::Content(String::from("testing"))],
                }
            ])
        );
    }

    #[test]
    fn extend_styled_and_content_token_with_content_token() {
        let mut tokens = Tokens::from(vec![
            Token::Styled {
                style: style!(),
                content: vec![Token::Content(String::from("existing styled"))],
            },
            Token::Content(String::from("existing content")),
        ]);
        tokens.extend(vec![Token::Content(String::from("testing"))]);
        assert_eq!(
            tokens,
            Tokens::from(vec![
                Token::Styled {
                    style: style!(),
                    content: vec![Token::Content(String::from("existing styled"))],
                },
                Token::Content(String::from("existing contenttesting")),
            ])
        );
    }

    #[test]
    fn extend_content_and_styled_token_with_content_token() {
        let mut tokens = Tokens::from(vec![
            Token::Content(String::from("existing content")),
            Token::Styled {
                style: style!(),
                content: vec![Token::Content(String::from("existing styled"))],
            },
        ]);
        tokens.extend(vec![Token::Content(String::from("testing"))]);
        assert_eq!(
            tokens,
            Tokens::from(vec![
                Token::Content(String::from("existing content")),
                Token::Styled {
                    style: style!(),
                    content: vec![Token::Content(String::from("existing styled"))],
                },
                Token::Content(String::from("testing"))
            ])
        );
    }
}
