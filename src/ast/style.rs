use super::{Color, Decoration};
use indexmap::IndexSet;

/// Styles that can be applied to a piece of text
#[derive(Clone, Debug, Default)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Style {
    /// The foreground color
    pub foreground: Option<Color>,
    /// The background color
    pub background: Option<Color>,
    /// Additional text decoration (i.e. bold, italic, underline, etc.)
    pub decoration: Option<IndexSet<Decoration>>,
}

impl Style {
    /// Check if the style has any properties
    fn is_empty(&self) -> bool {
        let has_decorations = match &self.decoration {
            Some(decorations) => decorations.is_empty(),
            None => true,
        };
        self.foreground.is_none() && self.background.is_none() && has_decorations
    }

    /// Apply the current style to the text
    pub fn apply(&self, parent: &CurrentStyle, output: &mut String) {
        if self.is_empty() {
            return;
        }

        // typically we'll only have a foreground and single decoration
        let mut codes = Vec::with_capacity(2);

        if let Some(foreground) = self.foreground {
            if foreground != parent.foreground {
                codes.push(foreground.foreground_code())
            }
        }

        if let Some(background) = self.background {
            if background != parent.background {
                codes.push(background.background_code());
            }
        }

        if let Some(decorations) = &self.decoration {
            codes.extend(
                decorations
                    .difference(&parent.decoration)
                    .map(Decoration::apply_code),
            );
        }

        self.append_codes(codes, output);
    }

    /// Reset the style to what it was previously
    pub fn reset(&self, parent: &CurrentStyle, output: &mut String) {
        if self.is_empty() {
            return;
        }

        // typically we'll only have a foreground and single decoration
        let mut codes = Vec::with_capacity(2);

        if let Some(foreground) = self.foreground {
            if foreground != parent.foreground {
                codes.push(parent.foreground.foreground_code());
            }
        }

        if let Some(background) = self.background {
            if background != parent.background {
                codes.push(parent.background.background_code());
            }
        }

        if let Some(decorations) = &self.decoration {
            codes.extend(
                decorations
                    .difference(&parent.decoration)
                    .map(Decoration::remove_code),
            );
        }

        self.append_codes(codes, output);
    }

    /// Append the ANSI codes to the output
    fn append_codes(&self, codes: Vec<&str>, output: &mut String) {
        if codes.is_empty() {
            return;
        }

        output.push_str("\x1b[");
        output.push_str(&codes.join(";"));
        output.push('m');
    }
}

/// The current styles applied to a piece of text
#[derive(Clone, Debug, Default)]
pub struct CurrentStyle {
    foreground: Color,
    background: Color,
    decoration: IndexSet<Decoration>,
}

impl CurrentStyle {
    /// Extend the current style with additional styles from a token
    pub fn extend(&self, style: &Style) -> CurrentStyle {
        let mut current = CurrentStyle::clone(self);

        current.foreground = style.foreground.unwrap_or(current.foreground);
        current.background = style.background.unwrap_or(current.background);
        if let Some(decoration) = &style.decoration {
            current.decoration.extend(decoration.iter());
        }

        current
    }
}

impl From<Style> for CurrentStyle {
    fn from(style: Style) -> Self {
        CurrentStyle {
            foreground: style.foreground.unwrap_or_default(),
            background: style.background.unwrap_or_default(),
            decoration: style.decoration.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CurrentStyle, Style};

    #[test]
    fn default_is_empty() {
        let style = style!();
        assert!(style.is_empty());
    }

    #[test]
    fn is_empty_with_foreground() {
        let style = style!(fg: Red;);
        assert!(!style.is_empty());
    }

    #[test]
    fn is_empty_with_background() {
        let style = style!(bg: Blue;);
        assert!(!style.is_empty());
    }

    #[test]
    fn is_empty_with_single_decoration() {
        let style = style!(deco: Bold;);
        assert!(!style.is_empty());
    }

    #[test]
    fn is_empty_with_multiple_decorations() {
        let style = style!(deco: Bold, Italic;);
        assert!(!style.is_empty());
    }

    /// Create a sequence of tests
    macro_rules! simple_tests {
    (
        for $function:ident;
        $( $name:ident: $style:expr, $parent:expr => $output:expr ),* $(,)?
    ) => {
            $(
                #[test]
                fn $name () {
                    assert_eq!($function(&$style, $parent), $output);
                }
            )*
        };
    }

    fn apply(style: &Style, parent: Style) -> String {
        let mut output = String::new();
        style.apply(&parent.into(), &mut output);
        output
    }

    #[test]
    fn apply_appends_to_end_of_output() {
        let mut output = String::from("existing");

        let style = style!(fg: Red;);
        style.apply(&CurrentStyle::default(), &mut output);

        assert_eq!(output, "existing\x1b[31m");
    }

    simple_tests! {
        for apply;

        apply_foreground_different_from_parent: style!(fg: Red;), style!(fg: Blue;) => "\x1b[31m",
        apply_foreground_identical_to_parent: style!(fg: Red;), style!(fg: Red;) => "",
        apply_background_different_from_parent: style!(bg: Red;), style!(bg: Blue;) => "\x1b[41m",
        apply_background_identical_to_parent: style!(bg: Red;), style!(bg: Red;) => "",
        apply_single_decoration_different_from_parent: style!(deco: Bold;), style!(deco: Dim;) => "\x1b[1m",
        apply_multiple_decoration_different_from_parent: style!(deco: Bold, Italic;), style!(deco: Dim, Hide;) => "\x1b[1;3m",
        apply_single_decoration_identical_to_parent: style!(deco: Bold;), style!(deco: Bold;) => "",
        apply_multiple_decorations_identical_to_parent: style!(deco: Bold, Italic;), style!(deco: Bold, Italic;) => "",

        apply_foreground_and_background_no_parent: style!(fg: Red; bg: Blue;), style!() => "\x1b[31;44m",
        apply_foreground_and_single_decoration_no_parent: style!(fg: Red; deco: Bold;), style!() => "\x1b[31;1m",
        apply_foreground_and_multiple_decorations_no_parent: style!(fg: Red; deco: Bold, Italic;), style!() => "\x1b[31;1;3m",
        apply_background_and_single_decoration_no_parent: style!(bg: Red; deco: Bold;), style!() => "\x1b[41;1m",
        apply_background_and_mulitple_decoration_no_parent: style!(bg: Red; deco: Bold, Italic;), style!() => "\x1b[41;1;3m",

        apply_foreground_and_background_parent_matches_neither: style!(fg: Red; bg: Blue;), style!(deco: Bold;) => "\x1b[31;44m",
        apply_foreground_and_background_parent_matches_foreground: style!(fg: Red; bg: Blue;), style!(fg: Red;) => "\x1b[44m",
        apply_foreground_and_background_parent_matches_background: style!(fg: Red; bg: Blue;), style!(bg: Blue;) => "\x1b[31m",
        apply_foreground_and_background_parent_matches_both: style!(fg: Red; bg: Blue;), style!(fg: Red; bg: Blue;) => "",

        apply_foreground_and_single_decoration_parent_matches_neither: style!(fg: Red; deco: Bold;), style!(bg: Blue;) => "\x1b[31;1m",
        apply_foreground_and_single_decoration_parent_matches_foreground: style!(fg: Red; deco: Bold;), style!(fg: Red;) => "\x1b[1m",
        apply_foreground_and_single_decoration_parent_matches_decoration: style!(fg: Red; deco: Bold;), style!(deco: Bold;) => "\x1b[31m",
        apply_foreground_and_single_decoration_parent_matches_both: style!(fg: Red; deco: Bold;), style!(fg: Red; deco: Bold;) => "",

        apply_foreground_and_multiple_decorations_parent_matches_neither: style!(fg: Red; deco: Bold, Italic;), style!(bg: Blue;) => "\x1b[31;1;3m",
        apply_foreground_and_multiple_decorations_parent_matches_foreground: style!(fg: Red; deco: Bold, Italic;), style!(fg: Red;) => "\x1b[1;3m",
        apply_foreground_and_multiple_decorations_parent_matches_first_decoration: style!(fg: Red; deco: Bold, Italic;), style!(deco: Bold;) => "\x1b[31;3m",
        apply_foreground_and_multiple_decorations_parent_matches_second_decoration: style!(fg: Red; deco: Bold, Italic;), style!(deco: Italic;) => "\x1b[31;1m",
        apply_foreground_and_mulitple_decorations_parent_matches_both_decorations: style!(fg: Red; deco: Bold, Italic;), style!(deco: Bold, Italic;) => "\x1b[31m",

        apply_background_and_single_decoration_parent_matches_neither: style!(bg: Blue; deco: Bold;), style!(fg: Red;) => "\x1b[44;1m",
        apply_background_and_single_decoration_parent_matches_foreground: style!(bg: Blue; deco: Bold;), style!(bg: Blue;) => "\x1b[1m",
        apply_background_and_single_decoration_parent_matches_decoration: style!(bg: Blue; deco: Bold;), style!(deco: Bold;) => "\x1b[44m",
        apply_background_and_single_decoration_parent_matches_both: style!(bg: Blue; deco: Bold;), style!(bg: Blue; deco: Bold;) => "",

        apply_background_and_multiple_decorations_parent_matches_neither: style!(bg: Blue; deco: Bold, Italic;), style!(fg: Red;) => "\x1b[44;1;3m",
        apply_background_and_multiple_decorations_parent_matches_foreground: style!(bg: Blue; deco: Bold, Italic;), style!(bg: Blue;) => "\x1b[1;3m",
        apply_background_and_multiple_decorations_parent_matches_first_decoration: style!(bg: Blue; deco: Bold, Italic;), style!(deco: Bold;) => "\x1b[44;3m",
        apply_background_and_multiple_decorations_parent_matches_second_decoration: style!(bg: Blue; deco: Bold, Italic;), style!(deco: Italic;) => "\x1b[44;1m",
        apply_background_and_mulitple_decorations_parent_matches_both_decorations: style!(bg: Blue; deco: Bold, Italic;), style!(deco: Bold, Italic;) => "\x1b[44m",
    }

    simple_tests! {
        for apply;

        ansi_code_foreground_default: style!(fg: Default;), style!(fg: Red;) => "\x1b[39m",
        ansi_code_background_default: style!(bg: Default;), style!(bg: Red;) => "\x1b[49m",

        ansi_code_foreground_black: style!(fg: Black;), style!() => "\x1b[30m",
        ansi_code_foreground_red: style!(fg: Red;), style!() => "\x1b[31m",
        ansi_code_foreground_green: style!(fg: Green;), style!() => "\x1b[32m",
        ansi_code_foreground_yellow: style!(fg: Yellow;), style!() => "\x1b[33m",
        ansi_code_foreground_blue: style!(fg: Blue;), style!() => "\x1b[34m",
        ansi_code_foreground_magenta: style!(fg: Magenta;), style!() => "\x1b[35m",
        ansi_code_foreground_cyan: style!(fg: Cyan;), style!() => "\x1b[36m",
        ansi_code_foreground_white: style!(fg: White;), style!() => "\x1b[37m",
        ansi_code_foreground_bright_black: style!(fg: BrightBlack;), style!() => "\x1b[90m",
        ansi_code_foreground_bright_red: style!(fg: BrightRed;), style!() => "\x1b[91m",
        ansi_code_foreground_bright_green: style!(fg: BrightGreen;), style!() => "\x1b[92m",
        ansi_code_foreground_bright_yellow: style!(fg: BrightYellow;), style!() => "\x1b[93m",
        ansi_code_foreground_bright_blue: style!(fg: BrightBlue;), style!() => "\x1b[94m",
        ansi_code_foreground_bright_magenta: style!(fg: BrightMagenta;), style!() => "\x1b[95m",
        ansi_code_foreground_bright_cyan: style!(fg: BrightCyan;), style!() => "\x1b[96m",
        ansi_code_foreground_bright_white: style!(fg: BrightWhite;), style!() => "\x1b[97m",

        ansi_code_background_black: style!(bg: Black;), style!() => "\x1b[40m",
        ansi_code_background_red: style!(bg: Red;), style!() => "\x1b[41m",
        ansi_code_background_green: style!(bg: Green;), style!() => "\x1b[42m",
        ansi_code_background_yellow: style!(bg: Yellow;), style!() => "\x1b[43m",
        ansi_code_background_blue: style!(bg: Blue;), style!() => "\x1b[44m",
        ansi_code_background_magenta: style!(bg: Magenta;), style!() => "\x1b[45m",
        ansi_code_background_cyan: style!(bg: Cyan;), style!() => "\x1b[46m",
        ansi_code_background_white: style!(bg: White;), style!() => "\x1b[47m",
        ansi_code_background_bright_black: style!(bg: BrightBlack;), style!() => "\x1b[100m",
        ansi_code_background_bright_red: style!(bg: BrightRed;), style!() => "\x1b[101m",
        ansi_code_background_bright_green: style!(bg: BrightGreen;), style!() => "\x1b[102m",
        ansi_code_background_bright_yellow: style!(bg: BrightYellow;), style!() => "\x1b[103m",
        ansi_code_background_bright_blue: style!(bg: BrightBlue;), style!() => "\x1b[104m",
        ansi_code_background_bright_magenta: style!(bg: BrightMagenta;), style!() => "\x1b[105m",
        ansi_code_background_bright_cyan: style!(bg: BrightCyan;), style!() => "\x1b[106m",
        ansi_code_background_bright_white: style!(bg: BrightWhite;), style!() => "\x1b[107m",

        ansi_code_decoration_bold: style!(deco: Bold;), style!() => "\x1b[1m",
        ansi_code_decoration_dim: style!(deco: Dim;), style!() => "\x1b[2m",
        ansi_code_decoration_italic: style!(deco: Italic;), style!() => "\x1b[3m",
        ansi_code_decoration_underline: style!(deco: Underline;), style!() => "\x1b[4m",
        ansi_code_decoration_slow_blink: style!(deco: SlowBlink;), style!() => "\x1b[5m",
        ansi_code_decoration_fast_blink: style!(deco: FastBlink;), style!() => "\x1b[6m",
        ansi_code_decoration_invert: style!(deco: Invert;), style!() => "\x1b[7m",
        ansi_code_decoration_hide: style!(deco: Hide;), style!() => "\x1b[8m",
        ansi_code_decoration_strike_through: style!(deco: StrikeThrough;), style!() => "\x1b[9m",
    }

    fn reset(style: &Style, parent: Style) -> String {
        let mut output = String::new();
        style.reset(&parent.into(), &mut output);
        output
    }

    #[test]
    fn reset_appends_to_end_of_output() {
        let mut output = String::from("existing");

        let style = style!(fg: Red;);
        style.reset(&CurrentStyle::default(), &mut output);

        assert_eq!(output, "existing\x1b[39m");
    }

    simple_tests! {
        for reset;

        reset_foreground_different_from_parent: style!(fg: Red;), style!(fg: Blue;) => "\x1b[34m",
        reset_foreground_identical_to_parent: style!(fg: Red;), style!(fg: Red;) => "",
        reset_background_different_from_parent: style!(bg: Red;), style!(bg: Blue;) => "\x1b[44m",
        reset_background_identical_to_parent: style!(bg: Red;), style!(bg: Red;) => "",
        reset_single_decoration_different_from_parent: style!(deco: Bold;), style!(deco: Dim;) => "\x1b[22m",
        reset_multiple_decoration_different_from_parent: style!(deco: Bold, Italic;), style!(deco: Dim, Hide;) => "\x1b[22;23m",
        reset_single_decoration_identical_to_parent: style!(deco: Bold;), style!(deco: Bold;) => "",
        reset_multiple_decorations_identical_to_parent: style!(deco: Bold, Italic;), style!(deco: Bold, Italic;) => "",

        reset_foreground_and_background_no_parent: style!(fg: Red; bg: Blue;), style!() => "\x1b[39;49m",
        reset_foreground_and_single_decoration_no_parent: style!(fg: Red; deco: Bold;), style!() => "\x1b[39;22m",
        reset_foreground_and_multiple_decorations_no_parent: style!(fg: Red; deco: Bold, Italic;), style!() => "\x1b[39;22;23m",
        reset_background_and_single_decoration_no_parent: style!(bg: Red; deco: Bold;), style!() => "\x1b[49;22m",
        reset_background_and_mulitple_decoration_no_parent: style!(bg: Red; deco: Bold, Italic;), style!() => "\x1b[49;22;23m",

        reset_foreground_and_background_parent_matches_neither: style!(fg: Red; bg: Blue;), style!(deco: Bold;) => "\x1b[39;49m",
        reset_foreground_and_background_parent_matches_foreground: style!(fg: Red; bg: Blue;), style!(fg: Red;) => "\x1b[49m",
        reset_foreground_and_background_parent_matches_background: style!(fg: Red; bg: Blue;), style!(bg: Blue;) => "\x1b[39m",
        reset_foreground_and_background_parent_matches_both: style!(fg: Red; bg: Blue;), style!(fg: Red; bg: Blue;) => "",

        reset_foreground_and_single_decoration_parent_matches_neither: style!(fg: Red; deco: Bold;), style!(bg: Blue;) => "\x1b[39;22m",
        reset_foreground_and_single_decoration_parent_matches_foreground: style!(fg: Red; deco: Bold;), style!(fg: Red;) => "\x1b[22m",
        reset_foreground_and_single_decoration_parent_matches_decoration: style!(fg: Red; deco: Bold;), style!(deco: Bold;) => "\x1b[39m",
        reset_foreground_and_single_decoration_parent_matches_both: style!(fg: Red; deco: Bold;), style!(fg: Red; deco: Bold;) => "",

        reset_foreground_and_multiple_decorations_parent_matches_neither: style!(fg: Red; deco: Bold, Italic;), style!(bg: Blue;) => "\x1b[39;22;23m",
        reset_foreground_and_multiple_decorations_parent_matches_foreground: style!(fg: Red; deco: Bold, Italic;), style!(fg: Red;) => "\x1b[22;23m",
        reset_foreground_and_multiple_decorations_parent_matches_first_decoration: style!(fg: Red; deco: Bold, Italic;), style!(deco: Bold;) => "\x1b[39;23m",
        reset_foreground_and_multiple_decorations_parent_matches_second_decoration: style!(fg: Red; deco: Bold, Italic;), style!(deco: Italic;) => "\x1b[39;22m",
        reset_foreground_and_mulitple_decorations_parent_matches_both_decorations: style!(fg: Red; deco: Bold, Italic;), style!(deco: Bold, Italic;) => "\x1b[39m",

        reset_background_and_single_decoration_parent_matches_neither: style!(bg: Blue; deco: Bold;), style!(fg: Red;) => "\x1b[49;22m",
        reset_background_and_single_decoration_parent_matches_foreground: style!(bg: Blue; deco: Bold;), style!(bg: Blue;) => "\x1b[22m",
        reset_background_and_single_decoration_parent_matches_decoration: style!(bg: Blue; deco: Bold;), style!(deco: Bold;) => "\x1b[49m",
        reset_background_and_single_decoration_parent_matches_both: style!(bg: Blue; deco: Bold;), style!(bg: Blue; deco: Bold;) => "",

        reset_background_and_multiple_decorations_parent_matches_neither: style!(bg: Blue; deco: Bold, Italic;), style!(fg: Red;) => "\x1b[49;22;23m",
        reset_background_and_multiple_decorations_parent_matches_foreground: style!(bg: Blue; deco: Bold, Italic;), style!(bg: Blue;) => "\x1b[22;23m",
        reset_background_and_multiple_decorations_parent_matches_first_decoration: style!(bg: Blue; deco: Bold, Italic;), style!(deco: Bold;) => "\x1b[49;23m",
        reset_background_and_multiple_decorations_parent_matches_second_decoration: style!(bg: Blue; deco: Bold, Italic;), style!(deco: Italic;) => "\x1b[49;22m",
        reset_background_and_mulitple_decorations_parent_matches_both_decorations: style!(bg: Blue; deco: Bold, Italic;), style!(deco: Bold, Italic;) => "\x1b[49m",
    }
}
