use crate::styles::{Color, Decoration};
use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::multispace0,
    combinator::value,
    error::{context, ContextError, ErrorKind, ParseError},
    sequence::delimited,
    AsChar, Compare, IResult, InputIter, InputLength, InputTake, InputTakeAtPosition, Parser,
    Slice,
};
use std::ops::RangeFrom;

/// Parse a [`Color`]
pub(crate) fn color<I, E>(input: I) -> IResult<I, Color, E>
where
    I: Clone + InputTake + Compare<&'static str>,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "color",
        alt((
            value(Color::Black, tag_no_case("black")),
            value(Color::Red, tag_no_case("red")),
            value(Color::Green, tag_no_case("green")),
            value(Color::Yellow, tag_no_case("yellow")),
            value(Color::Blue, tag_no_case("blue")),
            value(Color::Magenta, tag_no_case("magenta")),
            value(Color::Cyan, tag_no_case("cyan")),
            value(Color::White, tag_no_case("white")),
            value(Color::Default, tag_no_case("default")),
            value(Color::BrightBlack, tag_no_case("bright-black")),
            value(Color::BrightRed, tag_no_case("bright-red")),
            value(Color::BrightGreen, tag_no_case("bright-green")),
            value(Color::BrightYellow, tag_no_case("bright-yellow")),
            value(Color::BrightBlue, tag_no_case("bright-blue")),
            value(Color::BrightMagenta, tag_no_case("bright-magenta")),
            value(Color::BrightCyan, tag_no_case("bright-cyan")),
            value(Color::BrightWhite, tag_no_case("bright-white")),
        )),
    )(input)
}

/// Parse a text [`Decoration`]
pub(crate) fn decoration<I, E>(input: I) -> IResult<I, Decoration, E>
where
    I: Clone + InputTake + Compare<&'static str>,
    E: ParseError<I> + ContextError<I>,
{
    context(
        "decoration",
        alt((
            value(Decoration::Bold, tag_no_case("bold")),
            value(Decoration::Dim, tag_no_case("dim")),
            value(Decoration::Dim, tag_no_case("faint")),
            value(Decoration::Italic, tag_no_case("italic")),
            value(Decoration::Underline, tag_no_case("underline")),
            value(Decoration::SlowBlink, tag_no_case("slow-blink")),
            value(Decoration::FastBlink, tag_no_case("fast-blink")),
            value(Decoration::Invert, tag_no_case("invert")),
            value(Decoration::Invert, tag_no_case("reverse")),
            value(Decoration::Hide, tag_no_case("hide")),
            value(Decoration::Hide, tag_no_case("conceal")),
            value(Decoration::StrikeThrough, tag_no_case("strikethrough")),
            value(Decoration::StrikeThrough, tag_no_case("strike-through")),
        )),
    )(input)
}

/// Eats surrounding whitespace, producing the output of the `inner` parser.
pub(crate) fn whitespace<'a, F, I, O, E>(inner: F) -> impl Parser<I, O, E>
where
    F: Parser<I, O, E>,
    I: Clone + InputIter + InputLength + InputTake + InputTakeAtPosition,
    <I as InputIter>::Item: AsChar + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    E: ParseError<I>,
{
    delimited(multispace0, inner, multispace0)
}

/// Returns an error if the input was empty, otherwise consumes the entire input
pub(crate) fn non_empty<I, E>(input: I) -> IResult<I, I, E>
where
    I: InputLength + Slice<RangeFrom<usize>>,
    E: ParseError<I>,
{
    let len = input.input_len();
    match len {
        0 => Err(nom::Err::Error(E::from_error_kind(
            input,
            ErrorKind::NonEmpty,
        ))),
        _ => Ok((input.slice(len..), input)),
    }
}

#[cfg(test)]
mod tests {
    mod color {
        use crate::styles::Color;
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(color -> Color);

        simple_tests! {
            for color;
            black:          "black" => Color::Black,
            red:            "red" => Color::Red,
            green:          "green" => Color::Green,
            yellow:         "yellow" => Color::Yellow,
            blue:           "blue" => Color::Blue,
            magenta:        "magenta" => Color::Magenta,
            cyan:           "cyan" => Color::Cyan,
            white:          "white" => Color::White,
            bright_black:   "bright-black" => Color::BrightBlack,
            bright_red:     "bright-red" => Color::BrightRed,
            bright_green:   "bright-green" => Color::BrightGreen,
            bright_yellow:  "bright-yellow" => Color::BrightYellow,
            bright_blue:    "bright-blue" => Color::BrightBlue,
            bright_magenta: "bright-magenta" => Color::BrightMagenta,
            bright_cyan:    "bright-cyan" => Color::BrightCyan,
            bright_white:   "bright-white" => Color::BrightWhite,
            default:        "default" => Color::Default,
            lowercase:      "black" => Color::Black,
            uppercase:      "BLACK" => Color::Black,
            mixed_case:     "BlAcK" => Color::Black,
        }

        #[test]
        fn invalid() {
            assert_eq!(
                color("pink"),
                Err(nom::Err::Error(error_position!("pink", ErrorKind::Tag)))
            );
        }

        #[test]
        fn empty() {
            assert_eq!(
                color(""),
                Err(nom::Err::Error(error_position!("", ErrorKind::Tag)))
            )
        }
    }

    mod decoration {
        use crate::styles::Decoration;
        use nom::{error::ErrorKind, error_position};

        make_error_concrete!(decoration -> Decoration);

        simple_tests! {
            for decoration;
            bold:           "bold" => Decoration::Bold,
            dim:            "dim" => Decoration::Dim,
            faint:          "faint" => Decoration::Dim,
            italic:         "italic" => Decoration::Italic,
            underline:      "underline" => Decoration::Underline,
            slow_blink:     "slow-blink" => Decoration::SlowBlink,
            fast_blink:     "fast-blink" => Decoration::FastBlink,
            invert:         "invert" => Decoration::Invert,
            reverse:        "reverse" => Decoration::Invert,
            hide:           "hide" => Decoration::Hide,
            conceal:        "conceal" => Decoration::Hide,
            strikethrough:  "strikethrough" => Decoration::StrikeThrough,
            strike_through: "strike-through" => Decoration::StrikeThrough,
        }

        #[test]
        fn invalid() {
            assert_eq!(
                decoration("dunderline"),
                Err(nom::Err::Error(error_position!(
                    "dunderline",
                    ErrorKind::Tag
                )))
            );
        }

        #[test]
        fn empty() {
            assert_eq!(
                decoration(""),
                Err(nom::Err::Error(error_position!("", ErrorKind::Tag)))
            )
        }
    }
}
