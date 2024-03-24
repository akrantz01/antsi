use super::Parser;
use crate::{ast::Tokens, lexer::SyntaxKind};

/// Parse a piece of text that may content styled markup
pub(crate) fn text(p: &mut Parser) -> Tokens {
    let mut tokens = Tokens::default();

    loop {
        match p.peek() {
            Some(
                SyntaxKind::ParenthesisClose
                | SyntaxKind::ParenthesisOpen
                | SyntaxKind::SquareBracketClose,
            ) => break,
            Some(SyntaxKind::SquareBracketOpen) => todo!("handle nested"),
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

    tokens
}

#[cfg(test)]
mod tests {
    use super::{text, Parser};
    use crate::{
        ast::{Token, Tokens},
        lexer::SyntaxKind,
    };

    #[test]
    fn text_empty() {
        let mut parser = Parser::new("");
        assert_eq!(text(&mut parser), Tokens::from(vec![]));
    }

    #[test]
    fn text_stops_consuming_at_open_parenthesis() {
        let mut parser = Parser::new("before(after");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("before"))])
        );
        assert!(parser.at(SyntaxKind::ParenthesisOpen));
    }

    #[test]
    fn text_stops_consuming_at_close_parenthesis() {
        let mut parser = Parser::new("before)after");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("before"))])
        );
        assert!(parser.at(SyntaxKind::ParenthesisClose));
    }

    #[test]
    fn text_stops_consuming_at_close_square_bracket() {
        let mut parser = Parser::new("before]after");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("before"))])
        );
        assert!(parser.at(SyntaxKind::SquareBracketClose));
    }

    #[test]
    fn text_lowercase_alphabetic() {
        let mut parser = Parser::new("abcdefghijklmnopqrstuvwxyz");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from(
                "abcdefghijklmnopqrstuvwxyz"
            ))])
        )
    }

    #[test]
    fn text_uppercase_alphabetic() {
        let mut parser = Parser::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            ))])
        )
    }

    #[test]
    fn text_mixed_case_alphabetic() {
        let mut parser = Parser::new("AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYuZz");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from(
                "AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYuZz"
            ))])
        )
    }

    #[test]
    fn text_special_characters() {
        let mut parser = Parser::new("~!@#$%^&*-=_+~");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("~!@#$%^&*-=_+~"))])
        )
    }

    #[test]
    fn text_whitespace() {
        let mut parser = Parser::new(" \n\t\r");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from(" \n\t\r"))])
        )
    }

    #[test]
    fn text_matching_color() {
        let mut parser = Parser::new("black");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("black"))])
        );
    }

    #[test]
    fn text_matching_bright_color() {
        let mut parser = Parser::new("bright-blue");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("bright-blue"))])
        );
    }

    #[test]
    fn text_matching_default_color() {
        let mut parser = Parser::new("default");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("default"))])
        );
    }

    #[test]
    fn text_matching_decoration() {
        let mut parser = Parser::new("fast-blink");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("fast-blink"))])
        )
    }

    #[test]
    fn text_containing_colon() {
        let mut parser = Parser::new(":");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from(":"))])
        )
    }

    #[test]
    fn text_containing_semicolon() {
        let mut parser = Parser::new(";");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from(";"))])
        )
    }

    #[test]
    fn text_containing_comma() {
        let mut parser = Parser::new(",");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from(","))])
        )
    }

    #[test]
    fn text_containing_foreground_specifier() {
        let mut parser = Parser::new("fg");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("fg"))])
        );
    }

    #[test]
    fn text_containing_background_specifier() {
        let mut parser = Parser::new("bg");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("bg"))])
        );
    }

    #[test]
    fn text_containing_decoration_specifier() {
        let mut parser = Parser::new("deco");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("deco"))])
        );
    }

    #[test]
    fn text_escaped_backslash() {
        let mut parser = Parser::new("\\\\");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("\\"))])
        )
    }

    #[test]
    fn text_escaped_open_square_bracket() {
        let mut parser = Parser::new("\\[");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("["))])
        )
    }

    #[test]
    fn text_escaped_close_square_bracket() {
        let mut parser = Parser::new("\\]");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("]"))])
        )
    }

    #[test]
    fn text_escaped_open_parenthesis() {
        let mut parser = Parser::new("\\(");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from("("))])
        )
    }

    #[test]
    fn text_escaped_close_parenthesis() {
        let mut parser = Parser::new("\\)");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from(")"))])
        )
    }

    #[test]
    fn text_escaped_whitespace() {
        let mut parser = Parser::new("\\ \n\t\r");
        assert_eq!(text(&mut parser), Tokens::from(vec![]));
    }

    #[test]
    fn text_multiple_distinct_tokens() {
        let mut parser = Parser::new("some plaintext \\(ascii\\] \\\n\n :+1:");
        assert_eq!(
            text(&mut parser),
            Tokens::from(vec![Token::Content(String::from(
                "some plaintext (ascii] :+1:"
            ))])
        );
    }
}
