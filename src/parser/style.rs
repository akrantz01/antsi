use super::{ParseErrorReason, Parser};
use crate::{
    ast::{Color, Decoration, Style},
    lexer::SyntaxKind,
};
use indexmap::IndexSet;
use std::str::FromStr;

/// Extract style information from the token stream
pub(crate) fn style(p: &mut Parser) -> Option<Style> {
    p.expect(SyntaxKind::SquareBracketOpen)?;

    let mut style = Style::default();
    let mut first_specifier = true;

    loop {
        if !first_specifier {
            if p.at(SyntaxKind::Semicolon) {
                p.bump();
            } else {
                break;
            }
        }

        if p.at(SyntaxKind::ForegroundSpecifier) {
            let color = color_specifier(p, SyntaxKind::ForegroundSpecifier)?;
            style.foreground = Some(color);
        } else if p.at(SyntaxKind::BackgroundSpecifier) {
            let color = color_specifier(p, SyntaxKind::BackgroundSpecifier)?;
            style.background = Some(color);
        } else if p.at(SyntaxKind::DecorationSpecifier) {
            let decorations = decorations_specifier(p, SyntaxKind::DecorationSpecifier)?;
            style.decoration = Some(decorations);
        } else {
            p.error(ParseErrorReason::Expected(vec![
                SyntaxKind::ForegroundSpecifier,
                SyntaxKind::BackgroundSpecifier,
                SyntaxKind::DecorationSpecifier,
            ]));
            return None;
        };

        first_specifier = false;
    }

    p.expect(SyntaxKind::SquareBracketClose);

    Some(style)
}

/// Parse a specifier with a [`Color`] value
fn color_specifier(p: &mut Parser, tag: SyntaxKind) -> Option<Color> {
    p.expect(tag)?;
    p.expect(SyntaxKind::Colon)?;

    let token = p.expect(SyntaxKind::Color)?;
    Some(Color::from_str(token.text).expect("invalid color"))
}

/// Parse a specifier with a [`Decoration`]s value
fn decorations_specifier(p: &mut Parser, tag: SyntaxKind) -> Option<IndexSet<Decoration>> {
    p.expect(tag)?;
    p.expect(SyntaxKind::Colon)?;

    let mut decorations = IndexSet::with_capacity(1);
    let mut first_decoration = true;

    loop {
        if !first_decoration {
            if p.at(SyntaxKind::Comma) {
                p.bump();
            } else {
                break;
            }
        }

        let token = p.expect(SyntaxKind::Decoration)?;
        decorations.insert(Decoration::from_str(token.text).expect("invalid decoration"));

        first_decoration = false;
    }

    Some(decorations)
}

#[cfg(test)]
mod tests {
    use super::{color_specifier, decorations_specifier, style, Parser};
    use crate::{
        ast::{Color, Decoration},
        lexer::SyntaxKind,
    };

    #[test]
    fn foreground_color_specifier() {
        let mut parser = Parser::new("fg:red");
        let result = color_specifier(&mut parser, SyntaxKind::ForegroundSpecifier);
        assert_eq!(result, Some(Color::Red));
    }

    #[test]
    fn foreground_color_specifier_uppercase_tag() {
        let mut parser = Parser::new("FG:blue");
        let result = color_specifier(&mut parser, SyntaxKind::ForegroundSpecifier);
        assert_eq!(result, Some(Color::Blue));
    }

    #[test]
    fn foreground_color_specifier_uppercase_value() {
        let mut parser = Parser::new("fg:BLUE");
        let result = color_specifier(&mut parser, SyntaxKind::ForegroundSpecifier);
        assert_eq!(result, Some(Color::Blue));
    }

    #[test]
    fn foreground_color_specifier_all_uppercase() {
        let mut parser = Parser::new("FG:BLUE");
        let result = color_specifier(&mut parser, SyntaxKind::ForegroundSpecifier);
        assert_eq!(result, Some(Color::Blue));
    }

    #[test]
    fn background_color_specifier() {
        let mut parser = Parser::new("bg:red");
        let result = color_specifier(&mut parser, SyntaxKind::BackgroundSpecifier);
        assert_eq!(result, Some(Color::Red));
    }

    #[test]
    fn background_color_specifier_uppercase_tag() {
        let mut parser = Parser::new("BG:blue");
        let result = color_specifier(&mut parser, SyntaxKind::BackgroundSpecifier);
        assert_eq!(result, Some(Color::Blue));
    }

    #[test]
    fn background_color_specifier_uppercase_value() {
        let mut parser = Parser::new("bg:BLUE");
        let result = color_specifier(&mut parser, SyntaxKind::BackgroundSpecifier);
        assert_eq!(result, Some(Color::Blue));
    }

    #[test]
    fn background_color_specifier_all_uppercase() {
        let mut parser = Parser::new("BG:BLUE");
        let result = color_specifier(&mut parser, SyntaxKind::BackgroundSpecifier);
        assert_eq!(result, Some(Color::Blue));
    }

    #[test]
    fn decoration_specifier_single_decoration() {
        let mut parser = Parser::new("deco:bold");
        let result = decorations_specifier(&mut parser, SyntaxKind::DecorationSpecifier);
        assert_eq!(result, Some(set! { Decoration::Bold }));
    }

    #[test]
    fn decoration_specifier_two_decorations() {
        let mut parser = Parser::new("deco:bold,italic");
        let result = decorations_specifier(&mut parser, SyntaxKind::DecorationSpecifier);
        assert_eq!(result, Some(set! { Decoration::Bold, Decoration::Italic }));
    }

    #[test]
    fn decoration_specifier_many_decorations() {
        let mut parser = Parser::new("deco:bold,italic,hide,strike-through,fast-blink");
        let result = decorations_specifier(&mut parser, SyntaxKind::DecorationSpecifier);
        assert_eq!(
            result,
            Some(
                set! { Decoration::Bold, Decoration::Italic, Decoration::Hide, Decoration::StrikeThrough, Decoration::FastBlink }
            )
        );
    }

    #[test]
    fn decoration_specifier_duplicates_are_ignored() {
        let mut parser = Parser::new("deco:bold,bold");
        let result = decorations_specifier(&mut parser, SyntaxKind::DecorationSpecifier);
        assert_eq!(result, Some(set! { Decoration::Bold }));
    }

    #[test]
    fn decoration_specifier_interleaved_duplicates_are_ignored() {
        let mut parser = Parser::new("deco:bold,italic,bold,italic");
        let result = decorations_specifier(&mut parser, SyntaxKind::DecorationSpecifier);
        assert_eq!(result, Some(set! { Decoration::Bold, Decoration::Italic }));
    }

    #[test]
    fn decoration_specifier_uppercase_tag() {
        let mut parser = Parser::new("DECO:bold");
        let result = decorations_specifier(&mut parser, SyntaxKind::DecorationSpecifier);
        assert_eq!(result, Some(set! { Decoration::Bold }));
    }

    #[test]
    fn decoration_specifier_uppercase_value() {
        let mut parser = Parser::new("deco:BOLD");
        let result = decorations_specifier(&mut parser, SyntaxKind::DecorationSpecifier);
        assert_eq!(result, Some(set! { Decoration::Bold }));
    }

    #[test]
    fn decoration_specifier_all_uppercase() {
        let mut parser = Parser::new("DECO:BOLD");
        let result = decorations_specifier(&mut parser, SyntaxKind::DecorationSpecifier);
        assert_eq!(result, Some(set! { Decoration::Bold }));
    }

    #[test]
    fn style_foreground() {
        let mut parser = Parser::new("[fg:red]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(fg: Red;)));
    }

    #[test]
    fn style_background() {
        let mut parser = Parser::new("[bg:red]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(bg: Red;)));
    }

    #[test]
    fn style_single_decoration() {
        let mut parser = Parser::new("[deco:bold]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(deco: Bold;)));
    }

    #[test]
    fn style_multiple_decorations() {
        let mut parser = Parser::new("[deco:bold,italic]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(deco: Bold, Italic;)));
    }

    #[test]
    fn style_foreground_and_background() {
        let mut parser = Parser::new("[fg:red;bg:blue]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(fg: Red; bg: Blue;)));
    }

    #[test]
    fn style_foreground_and_single_decoration() {
        let mut parser = Parser::new("[fg:red;deco:bold]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(fg: Red; deco: Bold;)));
    }

    #[test]
    fn style_foreground_and_multiple_decorations() {
        let mut parser = Parser::new("[fg:red;deco:bold,italic]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(fg: Red; deco: Bold, Italic;)));
    }

    #[test]
    fn style_background_and_foreground() {
        let mut parser = Parser::new("[bg:blue;fg:red]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(bg: Blue; fg: Red;)));
    }

    #[test]
    fn style_background_and_single_decoration() {
        let mut parser = Parser::new("[bg:blue;deco:bold]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(bg: Blue; deco: Bold;)));
    }

    #[test]
    fn style_background_and_multiple_decorations() {
        let mut parser = Parser::new("[bg:blue;deco:bold,italic]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(bg: Blue; deco: Bold, Italic;)));
    }

    #[test]
    fn style_single_decoration_and_foreground() {
        let mut parser = Parser::new("[deco:bold;fg:red]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(deco: Bold; fg: Red;)));
    }

    #[test]
    fn style_multiple_decorations_and_foreground() {
        let mut parser = Parser::new("[deco:bold,italic;fg:red]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(deco: Bold, Italic; fg: Red;)));
    }

    #[test]
    fn style_single_decoration_and_background() {
        let mut parser = Parser::new("[deco:bold;bg:blue]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(deco: Bold; bg: Blue;)));
    }

    #[test]
    fn style_multiple_decorations_and_background() {
        let mut parser = Parser::new("[deco:bold,italic;bg:blue]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(deco: Bold, Italic; bg: Blue;)));
    }

    #[test]
    fn style_foreground_background_and_single_decoration() {
        let mut parser = Parser::new("[fg:red;bg:blue;deco:bold]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(fg: Red; bg: Blue; deco: Bold;)));
    }

    #[test]
    fn style_foreground_background_and_multiple_decorations() {
        let mut parser = Parser::new("[fg:red;bg:blue;deco:bold,italic]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(fg: Red; bg: Blue; deco: Bold, Italic;)));
    }

    #[test]
    fn style_foreground_single_decoration_and_background() {
        let mut parser = Parser::new("[fg:red;deco:bold;bg:blue]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(fg: Red; deco: Bold; bg: Blue;)));
    }

    #[test]
    fn style_foreground_multiple_decorations_and_background() {
        let mut parser = Parser::new("[fg:red;deco:bold,italic;bg:blue]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(fg: Red; deco: Bold, Italic; bg: Blue;)));
    }

    #[test]
    fn style_background_foreground_and_single_decoration() {
        let mut parser = Parser::new("[bg:blue;fg:red;deco:bold]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(bg: Blue; fg: Red; deco: Bold;)));
    }

    #[test]
    fn style_background_foreground_and_multiple_decorations() {
        let mut parser = Parser::new("[bg:blue;fg:red;deco:bold,italic]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(bg: Blue; fg: Red; deco: Bold, Italic;)));
    }

    #[test]
    fn style_background_single_decoration_and_foreground() {
        let mut parser = Parser::new("[bg:blue;deco:bold;fg:red]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(bg: Blue; deco: Bold; fg: Red;)));
    }

    #[test]
    fn style_background_multiple_decorations_and_foreground() {
        let mut parser = Parser::new("[bg:blue;deco:bold,italic;fg:red]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(bg: Blue; deco: Bold, Italic; fg: Red;)));
    }

    #[test]
    fn style_single_decoration_foreground_and_background() {
        let mut parser = Parser::new("[deco:bold;fg:red;bg:blue]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(deco: Bold; fg: Red; bg: Blue;)));
    }

    #[test]
    fn style_single_decoration_background_and_foreground() {
        let mut parser = Parser::new("[deco:bold;bg:blue;fg:red]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(deco: Bold; bg: Blue; fg: Red;)));
    }

    #[test]
    fn style_multiple_decorations_foreground_and_background() {
        let mut parser = Parser::new("[deco:bold,italic;fg:red;bg:blue]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(deco: Bold, Italic; fg: Red; bg: Blue;)));
    }

    #[test]
    fn style_multiple_decorations_background_and_foreground() {
        let mut parser = Parser::new("[deco:bold,italic;bg:blue;fg:red]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(deco: Bold, Italic; bg: Blue; fg: Red;)));
    }

    #[test]
    fn style_last_foreground_specifier_takes_precedence() {
        let mut parser = Parser::new("[fg:blue;fg:red]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(fg: Red;)));
    }

    #[test]
    fn style_last_background_specifier_takes_precedence() {
        let mut parser = Parser::new("[bg:red;bg:blue]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(bg: Blue;)));
    }

    #[test]
    fn style_last_decoration_specifier_takes_precedence() {
        let mut parser = Parser::new("[deco:italic;deco:bold]");
        let result = style(&mut parser);
        assert_eq!(result, Some(style!(deco: Bold;)));
    }
}
