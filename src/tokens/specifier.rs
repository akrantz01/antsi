use super::atoms::{color, decoration, whitespace};
use crate::ast::{Color, Decoration, Style};
use indexmap::IndexSet;
use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::char,
    combinator::{cut, map},
    error::{context, ContextError, ParseError},
    multi::separated_list1,
    sequence::{preceded, separated_pair, terminated},
    AsChar, Compare, IResult, InputIter, InputLength, InputTake, InputTakeAtPosition, Slice,
};
use std::ops::RangeFrom;

/// The value of an individual style specifier
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
enum StyleSpecifier {
    /// Apply a foreground color
    Foreground(Color),
    /// Apply a background color
    Background(Color),
    /// Apply a text decoration
    Decoration(IndexSet<Decoration>),
}

/// Parse a sequence of style specifiers
///
/// When duplicate specifiers are encountered, the last appearing one takes precedence
pub(crate) fn style<'a, I, E>(input: I) -> IResult<I, Style, E>
where
    I: Clone
        + Compare<&'static str>
        + InputIter
        + InputLength
        + InputTake
        + InputTakeAtPosition
        + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "style specifiers",
        preceded(
            char('['),
            cut(terminated(
                map(separated_list1(char(';'), style_specifier), |specifiers| {
                    specifiers
                        .into_iter()
                        .fold(Style::default(), |mut style, specifier| {
                            match specifier {
                                StyleSpecifier::Foreground(c) => style.foreground = Some(c),
                                StyleSpecifier::Background(c) => style.background = Some(c),
                                StyleSpecifier::Decoration(d) => style.decoration = Some(d),
                            }
                            style
                        })
                }),
                char(']'),
            )),
        ),
    )(input)
}

/// Parse a style specifier in the form `<key>:<value>`.
///
/// Depending on the value of the `<key>`, the value will be one of the following:
///   - Foreground color: `fg`   -> [`Color`]
///   - Background color: `bg`   -> [`Color`]
///   - Text decoration:  `deco` -> [`Decoration`]
fn style_specifier<I, E>(input: I) -> IResult<I, StyleSpecifier, E>
where
    I: Clone
        + Compare<&'static str>
        + InputIter
        + InputLength
        + InputTake
        + InputTakeAtPosition
        + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "style specifier",
        alt((
            foreground_specifier,
            background_specifier,
            decoration_specifier,
        )),
    )(input)
}

/// Parse a foreground [`Color`] specifier with the tag `fg`
fn foreground_specifier<I, E>(input: I) -> IResult<I, StyleSpecifier, E>
where
    I: Clone
        + Compare<&'static str>
        + InputIter
        + InputLength
        + InputTake
        + InputTakeAtPosition
        + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "foreground specifier",
        map(
            separated_pair(
                whitespace(tag_no_case("fg")),
                char(':'),
                cut(whitespace(color)),
            ),
            |(_, color)| StyleSpecifier::Foreground(color),
        ),
    )(input)
}

/// Parse a background [`Color`] specifier with the tag `bg`
fn background_specifier<I, E>(input: I) -> IResult<I, StyleSpecifier, E>
where
    I: Clone
        + Compare<&'static str>
        + InputIter
        + InputLength
        + InputTake
        + InputTakeAtPosition
        + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "background specifier",
        map(
            separated_pair(
                whitespace(tag_no_case("bg")),
                char(':'),
                cut(whitespace(color)),
            ),
            |(_, color)| StyleSpecifier::Background(color),
        ),
    )(input)
}

/// Parse a text [`Decoration`] specifier with the tag `deco`
fn decoration_specifier<I, E>(input: I) -> IResult<I, StyleSpecifier, E>
where
    I: Clone
        + Compare<&'static str>
        + InputIter
        + InputLength
        + InputTake
        + InputTakeAtPosition
        + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "decoration specifier",
        map(
            separated_pair(
                whitespace(tag_no_case("deco")),
                char(':'),
                separated_list1(char(','), cut(whitespace(decoration))),
            ),
            |(_, decorations)| StyleSpecifier::Decoration(IndexSet::from_iter(decorations)),
        ),
    )(input)
}

#[cfg(test)]
mod test {
    mod foreground_specifier {
        use super::super::StyleSpecifier;
        use crate::ast::Color;
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(foreground_specifier -> StyleSpecifier);

        simple_tests! {
            for foreground_specifier;
            black:          "fg:black" => StyleSpecifier::Foreground(Color::Black),
            red:            "fg:red" => StyleSpecifier::Foreground(Color::Red),
            green:          "fg:green" => StyleSpecifier::Foreground(Color::Green),
            yellow:         "fg:yellow" => StyleSpecifier::Foreground(Color::Yellow),
            blue:           "fg:blue" => StyleSpecifier::Foreground(Color::Blue),
            magenta:        "fg:magenta" => StyleSpecifier::Foreground(Color::Magenta),
            cyan:           "fg:cyan" => StyleSpecifier::Foreground(Color::Cyan),
            white:          "fg:white" => StyleSpecifier::Foreground(Color::White),
            default:        "fg:default" => StyleSpecifier::Foreground(Color::Default),
            bright_black:   "fg:bright-black" => StyleSpecifier::Foreground(Color::BrightBlack),
            bright_red:     "fg:bright-red" => StyleSpecifier::Foreground(Color::BrightRed),
            bright_green:   "fg:bright-green" => StyleSpecifier::Foreground(Color::BrightGreen),
            bright_yellow:  "fg:bright-yellow" => StyleSpecifier::Foreground(Color::BrightYellow),
            bright_blue:    "fg:bright-blue" => StyleSpecifier::Foreground(Color::BrightBlue),
            bright_magenta: "fg:bright-magenta" => StyleSpecifier::Foreground(Color::BrightMagenta),
            bright_cyan:    "fg:bright-cyan" => StyleSpecifier::Foreground(Color::BrightCyan),
            bright_white:   "fg:bright-white" => StyleSpecifier::Foreground(Color::BrightWhite),

            whitespace_ignored_on_tag: "  fg  :black" => StyleSpecifier::Foreground(Color::Black),
            whitespace_ignored_on_value: "fg:  black  " => StyleSpecifier::Foreground(Color::Black),
            whitespace_ignored_on_both_sides: "  fg  :  black  " => StyleSpecifier::Foreground(Color::Black),
        }

        #[test]
        fn invalid() {
            assert_eq!(
                foreground_specifier("fg:pink"),
                Err(nom::Err::Failure(error_position!("pink", ErrorKind::Tag)))
            );
        }
    }

    mod background_specifier {
        use super::super::StyleSpecifier;
        use crate::ast::Color;
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(background_specifier -> StyleSpecifier);

        simple_tests! {
            for background_specifier;
            black:          "bg:black" => StyleSpecifier::Background(Color::Black),
            red:            "bg:red" => StyleSpecifier::Background(Color::Red),
            green:          "bg:green" => StyleSpecifier::Background(Color::Green),
            yellow:         "bg:yellow" => StyleSpecifier::Background(Color::Yellow),
            blue:           "bg:blue" => StyleSpecifier::Background(Color::Blue),
            magenta:        "bg:magenta" => StyleSpecifier::Background(Color::Magenta),
            cyan:           "bg:cyan" => StyleSpecifier::Background(Color::Cyan),
            white:          "bg:white" => StyleSpecifier::Background(Color::White),
            default:        "bg:default" => StyleSpecifier::Background(Color::Default),
            bright_black:   "bg:bright-black" => StyleSpecifier::Background(Color::BrightBlack),
            bright_red:     "bg:bright-red" => StyleSpecifier::Background(Color::BrightRed),
            bright_green:   "bg:bright-green" => StyleSpecifier::Background(Color::BrightGreen),
            bright_yellow:  "bg:bright-yellow" => StyleSpecifier::Background(Color::BrightYellow),
            bright_blue:    "bg:bright-blue" => StyleSpecifier::Background(Color::BrightBlue),
            bright_magenta: "bg:bright-magenta" => StyleSpecifier::Background(Color::BrightMagenta),
            bright_cyan:    "bg:bright-cyan" => StyleSpecifier::Background(Color::BrightCyan),
            bright_white:   "bg:bright-white" => StyleSpecifier::Background(Color::BrightWhite),

            whitespace_ignored_on_tag:        "  bg  :black" => StyleSpecifier::Background(Color::Black),
            whitespace_ignored_on_value:      "bg:  black  " => StyleSpecifier::Background(Color::Black),
            whitespace_ignored_on_both_sides: "  bg  :  black  " => StyleSpecifier::Background(Color::Black),
        }

        #[test]
        fn invalid() {
            assert_eq!(
                background_specifier("bg:pink"),
                Err(nom::Err::Failure(error_position!("pink", ErrorKind::Tag)))
            );
        }
    }

    mod decoration_specifier {
        use super::super::StyleSpecifier;
        use crate::ast::Decoration;
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(decoration_specifier -> StyleSpecifier);

        simple_tests! {
            for decoration_specifier;
            bold:          "deco:bold" => StyleSpecifier::Decoration(set!{Decoration::Bold}),
            dim:           "deco:dim" => StyleSpecifier::Decoration(set!{Decoration::Dim}),
            italic:        "deco:italic" => StyleSpecifier::Decoration(set!{Decoration::Italic}),
            underline:     "deco:underline" => StyleSpecifier::Decoration(set!{Decoration::Underline}),
            slow_blink:    "deco:slow-blink" => StyleSpecifier::Decoration(set!{Decoration::SlowBlink}),
            fast_blink:    "deco:fast-blink" => StyleSpecifier::Decoration(set!{Decoration::FastBlink}),
            invert:        "deco:invert" => StyleSpecifier::Decoration(set!{Decoration::Invert}),
            hide:          "deco:hide" => StyleSpecifier::Decoration(set!{Decoration::Hide}),
            strikethrough: "deco:strikethrough" => StyleSpecifier::Decoration(set!{Decoration::StrikeThrough}),

            two_applied:   "deco:bold,dim" => StyleSpecifier::Decoration(set!{Decoration::Bold, Decoration::Dim}),
            three_applied: "deco:dim,bold,italic" => StyleSpecifier::Decoration(set!{Decoration::Dim, Decoration::Bold, Decoration::Italic}),

            duplicates_are_ignored: "deco:bold,bold" => StyleSpecifier::Decoration(set!{Decoration::Bold}),
            duplicates_of_different_kinds_are_ignored: "deco:bold,italic,bold,italic" => StyleSpecifier::Decoration(set!{Decoration::Bold, Decoration::Italic}),

            whitespace_ignored_on_tag:             "  deco  :bold" => StyleSpecifier::Decoration(set!{Decoration::Bold}),
            whitespace_ignored_on_value:           "deco:  bold  " => StyleSpecifier::Decoration(set!{Decoration::Bold}),
            whitespace_ignored_on_both_sides:      "  deco  :  bold  " => StyleSpecifier::Decoration(set!{Decoration::Bold}),
            whitespace_ignored_on_multiple_values: "deco:  bold  ,  italic  " => StyleSpecifier::Decoration(set!{Decoration::Bold, Decoration::Italic}),
        }

        #[test]
        fn invalid() {
            assert_eq!(
                decoration_specifier("deco:dunderline"),
                Err(nom::Err::Failure(error_position!(
                    "dunderline",
                    ErrorKind::Tag
                )))
            );
        }

        #[test]
        fn invalid_in_sequence() {
            assert_eq!(
                decoration_specifier("deco:bold,dunderline"),
                Err(nom::Err::Failure(error_position!(
                    "dunderline",
                    ErrorKind::Tag
                )))
            )
        }
    }

    mod style_specifier {
        use super::super::StyleSpecifier;
        use crate::ast::{Color, Decoration};
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(style_specifier -> StyleSpecifier);

        simple_tests! {
            for style_specifier;
            foreground: "fg:black" => StyleSpecifier::Foreground(Color::Black),
            background: "bg:black" => StyleSpecifier::Background(Color::Black),
            decoration: "deco:bold" => StyleSpecifier::Decoration(set!{Decoration::Bold}),
        }

        #[test]
        fn empty() {
            assert_eq!(
                style_specifier(""),
                Err(nom::Err::Error(error_position!("", ErrorKind::Tag)))
            )
        }

        #[test]
        fn unknown_tag() {
            assert_eq!(
                style_specifier("unknown:black"),
                Err(nom::Err::Error(error_position!(
                    "unknown:black",
                    ErrorKind::Tag
                )))
            )
        }

        #[test]
        fn missing_foreground_value() {
            assert_eq!(
                style_specifier("fg:"),
                Err(nom::Err::Failure(error_position!("", ErrorKind::Tag)))
            )
        }

        #[test]
        fn missing_background_value() {
            assert_eq!(
                style_specifier("bg:"),
                Err(nom::Err::Failure(error_position!("", ErrorKind::Tag)))
            )
        }

        #[test]
        fn missing_decoration_value() {
            assert_eq!(
                style_specifier("deco:"),
                Err(nom::Err::Failure(error_position!("", ErrorKind::Tag)))
            )
        }
    }

    mod style {
        use crate::tokens::Style;
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(style -> Style);

        simple_tests! {
            for style;
            foreground: "[fg:red]" => style!(fg: Red;),
            background: "[bg:red]" => style!(bg: Red;),
            decoration: "[deco:bold]" => style!(deco: Bold;),
            multiple_decorations: "[deco:bold,italic,fast-blink]" => style!(deco: Bold, Italic, FastBlink;),

            foreground_and_background: "[fg:red;bg:blue]" => style!(fg: Red; bg: Blue;),
            foreground_and_single_decoration: "[fg:red;deco:bold]" => style!(fg: Red; deco: Bold;),
            foreground_and_mutliple_decorations: "[fg:red;deco:bold,italic]" => style!(fg: Red; deco: Bold, Italic;),
            background_and_foreground: "[bg:blue;fg:red]" => style!(bg: Blue; fg: Red;),
            background_and_single_decoration: "[bg:blue;deco:bold]" => style!(bg: Blue; deco: Bold;),
            background_and_multiple_decorations: "[bg:blue;deco:bold,italic]" => style!(bg: Blue; deco: Bold, Italic;),
            single_decoration_and_foreground: "[deco:bold;fg:red]" => style!(deco: Bold; fg: Red;),
            multiple_decorations_and_foreground: "[deco:bold,italic;fg:red]" => style!(deco: Bold, Italic; fg: Red;),
            single_decoration_and_background: "[deco:bold;bg:blue]" => style!(deco: Bold; bg: Blue;),
            multiple_decorations_and_background: "[deco:bold,italic;bg:blue]" => style!(deco: Bold, Italic; bg: Blue;),

            foreground_background_and_single_decoration: "[fg:red;bg:blue;deco:bold]" => style!(fg: Red; bg: Blue; deco: Bold;),
            foreground_background_and_multiple_decorations: "[fg:red;bg:blue;deco:bold,italic]" => style!(fg: Red; bg: Blue; deco: Bold, Italic;),
            foreground_single_decoration_and_background: "[fg:red;deco:bold;bg:blue]" => style!(fg: Red; deco: Bold; bg: Blue;),
            foreground_multiple_decorations_and_background: "[fg:red;deco:bold,italic;bg:blue]" => style!(fg: Red; deco: Bold, Italic; bg: Blue;),
            background_foreground_and_single_decoration: "[bg:blue;fg:red;deco:bold]" => style!(bg: Blue; fg: Red; deco: Bold;),
            background_foreground_and_multiple_decorations: "[bg:blue;fg:red;deco:bold,italic]" => style!(bg: Blue; fg: Red; deco: Bold, Italic;),
            background_single_decoration_and_foreground: "[bg:blue;deco:bold;fg:red]" => style!(bg: Blue; deco: Bold; fg: Red;),
            background_multiple_decorations_and_foreground: "[bg:blue;deco:bold,italic;fg:red]" => style!(bg: Blue; deco: Bold, Italic; fg: Red;),
            single_decoration_foreground_and_background: "[deco:bold;fg:red;bg:blue]" => style!(deco: Bold; fg: Red; bg: Blue;),
            single_decoration_background_and_foreground: "[deco:bold;bg:blue;fg:red]" => style!(deco: Bold; bg: Blue; fg: Red;),
            multiple_decorations_foreground_and_background: "[deco:bold,italic;fg:red;bg:blue]" => style!(deco: Bold, Italic; fg: Red; bg: Blue;),
            multiple_decorations_background_and_foreground: "[deco:bold,italic;bg:blue;fg:red]" => style!(deco: Bold, Italic; bg: Blue; fg: Red;),
        }

        #[test]
        fn empty() {
            assert_eq!(
                style("[]"),
                Err(nom::Err::Failure(error_position!("]", ErrorKind::Tag)))
            )
        }

        #[test]
        fn invalid_tag() {
            assert_eq!(
                style("[ab:blue]"),
                Err(nom::Err::Failure(error_position!(
                    "ab:blue]",
                    ErrorKind::Tag
                )))
            )
        }

        #[test]
        fn invalid_foreground_color() {
            assert_eq!(
                style("[fg:pink]"),
                Err(nom::Err::Failure(error_position!("pink]", ErrorKind::Tag)))
            )
        }

        #[test]
        fn invalid_background_color() {
            assert_eq!(
                style("[bg:pink]"),
                Err(nom::Err::Failure(error_position!("pink]", ErrorKind::Tag)))
            )
        }

        #[test]
        fn invalid_text_decoration() {
            assert_eq!(
                style("[deco:dunderline]"),
                Err(nom::Err::Failure(error_position!(
                    "dunderline]",
                    ErrorKind::Tag
                )))
            )
        }
    }
}
