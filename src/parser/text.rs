use super::{markup::markup, Parser};
use crate::{ast::Tokens, lexer::SyntaxKind};

/// Parse a piece of text that may content styled markup
pub(crate) fn text(p: &mut Parser) -> Option<Tokens> {
    let mut tokens = Tokens::default();

    loop {
        match p.peek() {
            Some(
                SyntaxKind::ParenthesisClose
                | SyntaxKind::ParenthesisOpen
                | SyntaxKind::SquareBracketClose,
            ) => break,
            Some(SyntaxKind::SquareBracketOpen) => {
                let styled = markup(p)?;
                tokens.push(styled);
            }
            Some(SyntaxKind::EscapeWhitespace) => {
                p.bump();
            }
            Some(SyntaxKind::EscapeCharacter) => {
                let lexeme = p.bump();

                assert_eq!(lexeme.text.len(), 2);
                tokens.push_char(lexeme.text.chars().nth(1).unwrap());
            }
            Some(_) => {
                let lexeme = p.bump();
                tokens.push_str(lexeme.text);
            }
            None => break,
        }
    }

    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::{text, Parser};
    use crate::{
        ast::{Token, Tokens},
        lexer::SyntaxKind,
    };

    #[test]
    fn empty() {
        let mut parser = Parser::new("");
        assert_eq!(text(&mut parser), Some(Tokens::from(vec![])));
    }

    #[test]
    fn stops_consuming_at_open_parenthesis() {
        let mut parser = Parser::new("before(after");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("before"))]))
        );
        assert!(parser.at(SyntaxKind::ParenthesisOpen));
    }

    #[test]
    fn stops_consuming_at_close_parenthesis() {
        let mut parser = Parser::new("before)after");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("before"))]))
        );
        assert!(parser.at(SyntaxKind::ParenthesisClose));
    }

    #[test]
    fn stops_consuming_at_close_square_bracket() {
        let mut parser = Parser::new("before]after");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("before"))]))
        );
        assert!(parser.at(SyntaxKind::SquareBracketClose));
    }

    #[test]
    fn lowercase_alphabetic() {
        let mut parser = Parser::new("abcdefghijklmnopqrstuvwxyz");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "abcdefghijklmnopqrstuvwxyz"
            ))]))
        )
    }

    #[test]
    fn uppercase_alphabetic() {
        let mut parser = Parser::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            ))]))
        )
    }

    #[test]
    fn mixed_case_alphabetic() {
        let mut parser = Parser::new("AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYuZz");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYuZz"
            ))]))
        )
    }

    #[test]
    fn special_characters() {
        let mut parser = Parser::new("~!@#$%^&*-=_+~");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "~!@#$%^&*-=_+~"
            ))]))
        )
    }

    #[test]
    fn whitespace() {
        let mut parser = Parser::new(" \n\t\r");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(" \n\t\r"))]))
        )
    }

    #[test]
    fn matching_color() {
        let mut parser = Parser::new("black");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("black"))]))
        );
    }

    #[test]
    fn matching_bright_color() {
        let mut parser = Parser::new("bright-blue");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "bright-blue"
            ))]))
        );
    }

    #[test]
    fn matching_default_color() {
        let mut parser = Parser::new("default");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("default"))]))
        );
    }

    #[test]
    fn matching_decoration() {
        let mut parser = Parser::new("fast-blink");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "fast-blink"
            ))]))
        )
    }

    #[test]
    fn containing_colon() {
        let mut parser = Parser::new(":");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(":"))]))
        )
    }

    #[test]
    fn containing_semicolon() {
        let mut parser = Parser::new(";");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(";"))]))
        )
    }

    #[test]
    fn containing_comma() {
        let mut parser = Parser::new(",");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(","))]))
        )
    }

    #[test]
    fn containing_foreground_specifier() {
        let mut parser = Parser::new("fg");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("fg"))]))
        );
    }

    #[test]
    fn containing_background_specifier() {
        let mut parser = Parser::new("bg");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("bg"))]))
        );
    }

    #[test]
    fn containing_decoration_specifier() {
        let mut parser = Parser::new("deco");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("deco"))]))
        );
    }

    #[test]
    fn escaped_backslash() {
        let mut parser = Parser::new("\\\\");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("\\"))]))
        )
    }

    #[test]
    fn escaped_open_square_bracket() {
        let mut parser = Parser::new("\\[");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("["))]))
        )
    }

    #[test]
    fn escaped_close_square_bracket() {
        let mut parser = Parser::new("\\]");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("]"))]))
        )
    }

    #[test]
    fn escaped_open_parenthesis() {
        let mut parser = Parser::new("\\(");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("("))]))
        )
    }

    #[test]
    fn escaped_close_parenthesis() {
        let mut parser = Parser::new("\\)");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(")"))]))
        )
    }

    #[test]
    fn escaped_whitespace() {
        let mut parser = Parser::new("\\ \n\t\r");
        assert_eq!(text(&mut parser), Some(Tokens::from(vec![])));
    }

    #[test]
    fn multiple_distinct_tokens() {
        let mut parser = Parser::new("some plaintext \\(ascii\\] \\\n\n :+1:");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "some plaintext (ascii] :+1:"
            ))]))
        );
    }

    #[test]
    fn mixed_characters_and_escape_characters() {
        let mut parser = Parser::new("abc\\(DEF\\)12\\   34\\[!@#$\\]");
        assert_eq!(
            text(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "abc(DEF)1234[!@#$]"
            ))]))
        );
    }

    #[test]
    fn empty_token() {
        let mut parser = Parser::new("[fg:red]()");
        insta::assert_debug_snapshot!(text(&mut parser));
    }

    #[test]
    fn token_with_foreground() {
        let mut parser = Parser::new("[fg:red](inner)");
        insta::assert_debug_snapshot!(text(&mut parser));
    }

    #[test]
    fn token_with_background() {
        let mut parser = Parser::new("[bg:blue](inner)");
        insta::assert_debug_snapshot!(text(&mut parser));
    }

    #[test]
    fn token_with_single_decoration() {
        let mut parser = Parser::new("[deco:dim](inner)");
        insta::assert_debug_snapshot!(text(&mut parser));
    }

    #[test]
    fn token_with_multiple_decorations() {
        let mut parser = Parser::new("[deco:dim,italic](inner)");
        insta::assert_debug_snapshot!(text(&mut parser));
    }

    #[test]
    fn token_with_multiple_styles() {
        let mut parser = Parser::new("[deco:dim,italic;fg:red;bg:blue](inner)");
        insta::assert_debug_snapshot!(text(&mut parser));
    }

    #[test]
    fn token_with_leading_content() {
        let mut parser = Parser::new("leading [fg:red](content)");
        insta::assert_debug_snapshot!(text(&mut parser));
    }

    #[test]
    fn token_with_trailing_content() {
        let mut parser = Parser::new("[fg:red](content) trailing");
        insta::assert_debug_snapshot!(text(&mut parser));
    }

    #[test]
    fn token_with_leading_and_trailing_content() {
        let mut parser = Parser::new("leading [fg:red](content) trailing");
        insta::assert_debug_snapshot!(text(&mut parser));
    }

    #[test]
    fn nested_token() {
        let mut parser = Parser::new("[fg:red]([bg:blue](inner))");
        insta::assert_debug_snapshot!(text(&mut parser));
    }

    #[test]
    fn kitchen_sink() {
        let mut parser =
            Parser::new("leading [fg:red](one [bg:blue](two [deco:dim](three) two) one) trailing");
        insta::assert_debug_snapshot!(text(&mut parser));
    }
}
