use super::{text::text, Parser};
use crate::{ast::Tokens, lexer::SyntaxKind};

/// Parse a piece of styled content
pub(crate) fn content(p: &mut Parser) -> Option<Tokens> {
    p.expect(SyntaxKind::ParenthesisOpen)?;

    let tokens = text(p);

    p.expect(SyntaxKind::ParenthesisClose)?;

    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::{content, Parser};
    use crate::ast::{Token, Tokens};

    #[test]
    fn content_empty() {
        let mut parser = Parser::new("()");
        assert_eq!(content(&mut parser), Some(Tokens::from(vec![])));
    }

    #[test]
    fn content_lowercase_alphabetic() {
        let mut parser = Parser::new("(abcdefghijklmnopqrstuvwxyz)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "abcdefghijklmnopqrstuvwxyz"
            ))]))
        )
    }

    #[test]
    fn content_uppercase_alphabetic() {
        let mut parser = Parser::new("(ABCDEFGHIJKLMNOPQRSTUVWXYZ)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            ))]))
        )
    }

    #[test]
    fn content_mixed_case_alphabetic() {
        let mut parser = Parser::new("(AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYuZz)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYuZz"
            ))]))
        )
    }

    #[test]
    fn content_special_characters() {
        let mut parser = Parser::new("(~!@#$%^&*-=_+~)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "~!@#$%^&*-=_+~"
            ))]))
        )
    }

    #[test]
    fn content_whitespace() {
        let mut parser = Parser::new("( \n\t\r)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(" \n\t\r"))]))
        )
    }

    #[test]
    fn content_matching_color() {
        let mut parser = Parser::new("(black)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("black"))]))
        );
    }

    #[test]
    fn content_matching_bright_color() {
        let mut parser = Parser::new("(bright-blue)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "bright-blue"
            ))]))
        );
    }

    #[test]
    fn content_matching_default_color() {
        let mut parser = Parser::new("(default)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("default"))]))
        );
    }

    #[test]
    fn content_matching_decoration() {
        let mut parser = Parser::new("(fast-blink)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(
                "fast-blink"
            ))]))
        )
    }

    #[test]
    fn content_containing_colon() {
        let mut parser = Parser::new("(:)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(":"))]))
        )
    }

    #[test]
    fn content_containing_semicolon() {
        let mut parser = Parser::new("(;)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(";"))]))
        )
    }

    #[test]
    fn content_containing_comma() {
        let mut parser = Parser::new("(,)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(","))]))
        )
    }

    #[test]
    fn content_containing_foreground_specifier() {
        let mut parser = Parser::new("(fg)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("fg"))]))
        );
    }

    #[test]
    fn content_containing_background_specifier() {
        let mut parser = Parser::new("(bg)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("bg"))]))
        );
    }

    #[test]
    fn content_containing_decoration_specifier() {
        let mut parser = Parser::new("(deco)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("deco"))]))
        );
    }

    #[test]
    fn content_escaped_backslash() {
        let mut parser = Parser::new("(\\\\)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("\\"))]))
        )
    }

    #[test]
    fn content_escaped_open_square_bracket() {
        let mut parser = Parser::new("(\\[)");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("["))]))
        )
    }

    #[test]
    fn content_escaped_close_square_bracket() {
        let mut parser = Parser::new("(\\])");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("]"))]))
        )
    }

    #[test]
    fn content_escaped_open_parenthesis() {
        let mut parser = Parser::new("(\\()");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from("("))]))
        )
    }

    #[test]
    fn content_escaped_close_parenthesis() {
        let mut parser = Parser::new("(\\))");
        assert_eq!(
            content(&mut parser),
            Some(Tokens::from(vec![Token::Content(String::from(")"))]))
        )
    }

    #[test]
    fn content_escaped_whitespace() {
        let mut parser = Parser::new("(\\ \n\t\r)");
        assert_eq!(content(&mut parser), Some(Tokens::from(vec![])));
    }
}
