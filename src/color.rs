use crate::{
    ast::{CurrentStyle, Token},
    error::Error,
    parser::Parser,
};

pub fn colorize(input: &str) -> Result<String, Vec<Error>> {
    let (tokens, errors) = Parser::new(input).parse();
    if !errors.is_empty() {
        return Err(errors);
    }

    let mut result = String::with_capacity(input.len());
    convert_tokens(&mut result, CurrentStyle::default(), &tokens);

    result.shrink_to_fit();
    Ok(result)
}

/// Convert the tokens into the resulting string
fn convert_tokens(output: &mut String, parent_style: CurrentStyle, tokens: &[Token]) {
    for token in tokens {
        match token {
            Token::Content(content) => output.push_str(content),
            Token::Styled { content, style } => {
                if content.is_empty() {
                    continue;
                }

                style.apply(&parent_style, output);
                convert_tokens(output, parent_style.extend(style), content);
                style.reset(&parent_style, output);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::colorize;
    use crate::ast::{Style, Token};

    fn convert_tokens(parent_style: Option<Style>, tokens: &[Token]) -> String {
        let mut result = String::new();
        super::convert_tokens(&mut result, parent_style.unwrap_or_default().into(), tokens);
        result
    }

    #[test]
    fn convert_tokens_no_tokens() {
        let result = convert_tokens(None, &[]);
        assert_eq!(result, "");
    }

    #[test]
    fn convert_tokens_single_content_token() {
        let result = convert_tokens(None, &[Token::Content(String::from("test"))]);
        assert_eq!(result, "test");
    }

    #[test]
    fn convert_tokens_sequence_of_content_tokens() {
        let result = convert_tokens(
            None,
            &[
                Token::Content(String::from("a")),
                Token::Content(String::from("b")),
                Token::Content(String::from("c")),
            ],
        );

        assert_eq!(result, "abc");
    }

    #[test]
    fn convert_tokens_styled_token_with_no_style_or_content() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![],
                style: style!(),
            }],
        );
        assert_eq!(result, "");
    }

    #[test]
    fn convert_tokens_styled_token_with_no_style_and_single_content_token() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("test"))],
                style: style!(),
            }],
        );
        assert_eq!(result, "test")
    }

    #[test]
    fn convert_tokens_styled_token_with_no_style_and_sequence_of_content_tokens() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![
                    Token::Content(String::from("a")),
                    Token::Content(String::from("b")),
                    Token::Content(String::from("c")),
                ],
                style: style!(),
            }],
        );
        assert_eq!(result, "abc")
    }

    #[test]
    fn convert_tokens_styled_token_with_foreground_color() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(fg: Red;),
            }],
        );
        assert_eq!(result, "\x1b[31mcontent\x1b[39m");
    }

    #[test]
    fn convert_tokens_styled_token_with_background_color() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(bg: Blue;),
            }],
        );
        assert_eq!(result, "\x1b[44mcontent\x1b[49m");
    }

    #[test]
    fn convert_tokens_styled_token_with_single_decoration() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(deco: Bold;),
            }],
        );
        assert_eq!(result, "\x1b[1mcontent\x1b[22m");
    }

    #[test]
    fn convert_tokens_styled_token_with_multiple_decorations() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(deco: Bold, Italic;),
            }],
        );
        assert_eq!(result, "\x1b[1;3mcontent\x1b[22;23m");
    }

    #[test]
    fn convert_tokens_styled_token_with_foreground_and_background() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(fg: Red; bg: Blue;),
            }],
        );
        assert_eq!(result, "\x1b[31;44mcontent\x1b[39;49m");
    }

    #[test]
    fn convert_tokens_styled_token_with_foreground_and_single_decoration() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(fg: Red; deco: Bold;),
            }],
        );
        assert_eq!(result, "\x1b[31;1mcontent\x1b[39;22m");
    }

    #[test]
    fn convert_tokens_styled_token_with_foreground_and_multiple_decorations() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(fg: Red; deco: Bold, Italic;),
            }],
        );
        assert_eq!(result, "\x1b[31;1;3mcontent\x1b[39;22;23m");
    }

    #[test]
    fn convert_tokens_styled_token_with_background_and_foreground() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(bg: Blue; fg: Red;),
            }],
        );
        assert_eq!(result, "\x1b[31;44mcontent\x1b[39;49m");
    }

    #[test]
    fn convert_tokens_styled_token_with_background_and_single_decoration() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(bg: Blue; deco: Bold;),
            }],
        );
        assert_eq!(result, "\x1b[44;1mcontent\x1b[49;22m");
    }

    #[test]
    fn convert_tokens_styled_token_with_background_and_multiple_decorations() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(bg: Blue; deco: Bold, Italic;),
            }],
        );
        assert_eq!(result, "\x1b[44;1;3mcontent\x1b[49;22;23m");
    }

    #[test]
    fn convert_tokens_styled_token_with_single_decoration_and_foreground() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(deco: Bold; fg: Red;),
            }],
        );
        assert_eq!(result, "\x1b[31;1mcontent\x1b[39;22m");
    }

    #[test]
    fn convert_tokens_styled_token_with_single_decoration_and_background() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(deco: Bold; bg: Blue;),
            }],
        );
        assert_eq!(result, "\x1b[44;1mcontent\x1b[49;22m");
    }

    #[test]
    fn convert_tokens_styled_token_with_multiple_decorations_and_foreground() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(deco: Bold, Italic; fg: Red;),
            }],
        );
        assert_eq!(result, "\x1b[31;1;3mcontent\x1b[39;22;23m");
    }

    #[test]
    fn convert_tokens_styled_token_with_multiple_decorations_and_background() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![Token::Content(String::from("content"))],
                style: style!(deco: Bold, Italic; bg: Blue;),
            }],
        );
        assert_eq!(result, "\x1b[44;1;3mcontent\x1b[49;22;23m");
    }

    #[test]
    fn convert_tokens_style_token_with_nested_styling_non_overlapping() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![
                    Token::Content(String::from("red ")),
                    Token::Styled {
                        content: vec![Token::Content(String::from("blue"))],
                        style: style!(bg: Blue;),
                    },
                    Token::Content(String::from(" red")),
                ],
                style: style!(fg: Red;),
            }],
        );
        assert_eq!(result, "\x1b[31mred \x1b[44mblue\x1b[49m red\x1b[39m");
    }

    #[test]
    fn convert_tokens_style_token_with_nested_styling_overlapping_foreground() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![
                    Token::Content(String::from("red ")),
                    Token::Styled {
                        content: vec![Token::Content(String::from("blue"))],
                        style: style!(fg: Blue;),
                    },
                    Token::Content(String::from(" red")),
                ],
                style: style!(fg: Red;),
            }],
        );
        assert_eq!(result, "\x1b[31mred \x1b[34mblue\x1b[31m red\x1b[39m");
    }

    #[test]
    fn convert_tokens_style_token_with_nested_styling_overlapping_background() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![
                    Token::Content(String::from("red ")),
                    Token::Styled {
                        content: vec![Token::Content(String::from("blue"))],
                        style: style!(bg: Blue;),
                    },
                    Token::Content(String::from(" red")),
                ],
                style: style!(bg: Red;),
            }],
        );
        assert_eq!(result, "\x1b[41mred \x1b[44mblue\x1b[41m red\x1b[49m");
    }

    #[test]
    fn convert_tokens_style_token_with_nested_styling_overlapping_single_decoration() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![
                    Token::Content(String::from("bold ")),
                    Token::Styled {
                        content: vec![Token::Content(String::from("italic"))],
                        style: style!(deco: Italic;),
                    },
                    Token::Content(String::from(" bold")),
                ],
                style: style!(deco: Bold;),
            }],
        );
        assert_eq!(result, "\x1b[1mbold \x1b[3mitalic\x1b[23m bold\x1b[22m");
    }

    #[test]
    fn convert_tokens_style_token_with_nested_styling_overlapping_and_repeated_multiple_decoration()
    {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![
                    Token::Content(String::from("bold ")),
                    Token::Styled {
                        content: vec![Token::Content(String::from("italic"))],
                        style: style!(deco: Italic;),
                    },
                    Token::Content(String::from(" bold")),
                ],
                style: style!(deco: Bold, Italic;),
            }],
        );
        assert_eq!(result, "\x1b[1;3mbold italic bold\x1b[22;23m");
    }

    #[test]
    fn convert_tokens_style_token_with_nested_styling_repeated_foreground() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![
                    Token::Content(String::from("red ")),
                    Token::Styled {
                        content: vec![Token::Content(String::from("blue"))],
                        style: style!(fg: Red; bg: Blue;),
                    },
                    Token::Content(String::from(" red")),
                ],
                style: style!(fg: Red;),
            }],
        );
        assert_eq!(result, "\x1b[31mred \x1b[44mblue\x1b[49m red\x1b[39m");
    }

    #[test]
    fn convert_tokens_style_token_with_nested_styling_repeated_background() {
        let result = convert_tokens(
            None,
            &[Token::Styled {
                content: vec![
                    Token::Content(String::from("red ")),
                    Token::Styled {
                        content: vec![Token::Content(String::from("blue"))],
                        style: style!(fg: Red; bg: Blue;),
                    },
                    Token::Content(String::from(" red")),
                ],
                style: style!(bg: Blue;),
            }],
        );
        assert_eq!(result, "\x1b[44mred \x1b[31mblue\x1b[39m red\x1b[49m");
    }

    #[test]
    fn colorize_empty_source() {
        let result = colorize("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn colorize_source_with_no_styled_content() {
        let result = colorize("unstyled content").unwrap();
        assert_eq!(result, "unstyled content");
    }

    #[test]
    fn colorize_styled_content_spanning_entire_source() {
        let result = colorize("[fg:black](content)").unwrap();
        assert_eq!(result, "\x1b[30mcontent\x1b[39m");
    }

    #[test]
    fn colorize_styled_content_starting_at_beginning() {
        let result = colorize("[fg:black](content) unstyled").unwrap();
        assert_eq!(result, "\x1b[30mcontent\x1b[39m unstyled");
    }

    #[test]
    fn colorize_styled_content_starting_in_middle() {
        let result = colorize("leading [fg:black](content) trailing").unwrap();
        assert_eq!(result, "leading \x1b[30mcontent\x1b[39m trailing");
    }

    #[test]
    fn colorize_styled_content_starting_at_end() {
        let result = colorize("leading [fg:black](content)").unwrap();
        assert_eq!(result, "leading \x1b[30mcontent\x1b[39m");
    }

    #[test]
    fn colorize_styled_with_foreground() {
        let result = colorize("[fg:red](content)").unwrap();
        assert_eq!(result, "\x1b[31mcontent\x1b[39m");
    }

    #[test]
    fn colorize_styled_with_background() {
        let result = colorize("[bg:red](content)").unwrap();
        assert_eq!(result, "\x1b[41mcontent\x1b[49m");
    }

    #[test]
    fn colorize_styled_with_decoration() {
        let result = colorize("[deco:bold](content)").unwrap();
        assert_eq!(result, "\x1b[1mcontent\x1b[22m");
    }

    #[test]
    fn colorize_styled_with_foreground_and_background() {
        let result = colorize("[fg:red;bg:white](content)").unwrap();
        assert_eq!(result, "\x1b[31;47mcontent\x1b[39;49m");
    }

    #[test]
    fn colorize_styled_with_foreground_and_decoration() {
        let result = colorize("[fg:red;deco:bold](content)").unwrap();
        assert_eq!(result, "\x1b[31;1mcontent\x1b[39;22m");
    }

    #[test]
    fn colorize_styled_with_decoration_and_background() {
        let result = colorize("[deco:bold;bg:white](content)").unwrap();
        assert_eq!(result, "\x1b[47;1mcontent\x1b[49;22m");
    }

    #[test]
    fn colorize_styled_with_background_and_foreground() {
        let result = colorize("[bg:white;fg:red](content)").unwrap();
        assert_eq!(result, "\x1b[31;47mcontent\x1b[39;49m");
    }

    #[test]
    fn colorize_styled_with_decoration_and_foreground() {
        let result = colorize("[deco:bold;fg:red](content)").unwrap();
        assert_eq!(result, "\x1b[31;1mcontent\x1b[39;22m");
    }

    #[test]
    fn colorize_styled_with_background_and_decoration() {
        let result = colorize("[bg:white;deco:bold](content)").unwrap();
        assert_eq!(result, "\x1b[47;1mcontent\x1b[49;22m");
    }

    #[test]
    fn colorize_styled_with_foreground_background_and_decoration() {
        let result = colorize("[fg:red;bg:white;deco:bold](content)").unwrap();
        assert_eq!(result, "\x1b[31;47;1mcontent\x1b[39;49;22m");
    }

    #[test]
    fn colorize_styled_with_foreground_decoration_and_background() {
        let result = colorize("[fg:red;deco:bold;bg:white](content)").unwrap();
        assert_eq!(result, "\x1b[31;47;1mcontent\x1b[39;49;22m");
    }

    #[test]
    fn colorize_styled_with_background_foreground_and_decoration() {
        let result = colorize("[bg:white;fg:red;deco:bold](content)").unwrap();
        assert_eq!(result, "\x1b[31;47;1mcontent\x1b[39;49;22m");
    }

    #[test]
    fn colorize_styled_with_background_decoration_and_foreground() {
        let result = colorize("[bg:white;deco:bold;fg:red](content)").unwrap();
        assert_eq!(result, "\x1b[31;47;1mcontent\x1b[39;49;22m");
    }

    #[test]
    fn colorize_styled_with_decoration_foreground_and_background() {
        let result = colorize("[deco:bold;fg:red;bg:white](content)").unwrap();
        assert_eq!(result, "\x1b[31;47;1mcontent\x1b[39;49;22m");
    }

    #[test]
    fn colorize_styled_with_decoration_background_and_foreground() {
        let result = colorize("[deco:bold;bg:white;fg:red](content)").unwrap();
        assert_eq!(result, "\x1b[31;47;1mcontent\x1b[39;49;22m");
    }

    #[test]
    fn colorize_token_not_produced_for_content_with_zero_length() {
        let result = colorize("[fg:red]()").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn colorize_two_spans_of_styled_text_back_to_back() {
        let result = colorize("[fg:red](first)[fg:blue](second)").unwrap();
        assert_eq!(result, "\x1b[31mfirst\x1b[39m\x1b[34msecond\x1b[39m");
    }

    #[test]
    fn colorize_spans_of_styled_text_interleaved_with_unstyled_text() {
        let result =
            colorize("leading [fg:red](styled one) middle [fg:blue](styled two) trailing").unwrap();
        assert_eq!(
            result,
            "leading \x1b[31mstyled one\x1b[39m middle \x1b[34mstyled two\x1b[39m trailing"
        );
    }

    #[test]
    fn colorize_when_style_tags_are_repeated_the_last_occurrence_takes_precedence() {
        let result =
            colorize("[fg:black;bg:white;deco:bold;fg:white;bg:black;deco:faint](content)")
                .unwrap();
        assert_eq!(result, "\x1b[37;40;2mcontent\x1b[39;49;22m");
    }

    #[test]
    fn colorize_nested_styled_spans_produce_multiple_styled_tokens() {
        let result = colorize("user: [deco:bold](hi [fg:red](there)!)").unwrap();
        assert_eq!(result, "user: \x1b[1mhi \x1b[31mthere\x1b[39m!\x1b[22m");
    }
}
