use super::{content::content, style::style, Parser};
use crate::ast::Token;

/// Parse a segment of text with styling
pub(crate) fn markup(p: &mut Parser) -> Option<Token> {
    Some(Token::Styled {
        style: style(p)?,
        content: content(p)?.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::{markup, Parser};

    #[test]
    fn foreground_no_content() {
        let mut parser = Parser::new("[fg:red]()");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn background_no_content() {
        let mut parser = Parser::new("[bg:blue]()");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn decoration_single_style_no_content() {
        let mut parser = Parser::new("[deco:dim]()");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn decoration_multiple_styles_no_content() {
        let mut parser = Parser::new("[deco:dim,italic]()");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn lowercase_alphabetic_content() {
        let mut parser = Parser::new("[fg:red](hello)");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn uppercase_alphabetic_content() {
        let mut parser = Parser::new("[fg:red](HELLO)");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn mixed_alphabetic_content() {
        let mut parser = Parser::new("[fg:red](hElLo)");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn numeric_content() {
        let mut parser = Parser::new("[fg:red](12345)");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn special_character_content() {
        let mut parser = Parser::new("[fg:red](!@#$%^)");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn escaped_character_content() {
        let mut parser = Parser::new("[fg:red](\\(\\[\\]\\))");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn nested_token() {
        let mut parser = Parser::new("[fg:red]([bg:blue](inner))");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn nested_token_with_leading_content() {
        let mut parser = Parser::new("[fg:red](leading [bg:blue](inner))");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn nested_token_with_trailing_content() {
        let mut parser = Parser::new("[fg:red]([bg:blue](inner) trailing)");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }

    #[test]
    fn nested_token_with_leading_and_trailing_content() {
        let mut parser = Parser::new("[fg:red](leading [bg:blue](inner) trailing)");
        insta::assert_debug_snapshot!(markup(&mut parser));
    }
}
