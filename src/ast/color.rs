use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

/// The provided [`Color`] name was invalid
#[derive(Clone, Copy, Debug)]
pub struct InvalidColorError;

impl std::error::Error for InvalidColorError {}

impl Display for InvalidColorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid color name")
    }
}

macro_rules! colors {
    (
        $( $( #[ $meta:meta ] )* $color:ident $fg:literal $bg:literal ( $names:pat ) ),* $(,)?
    ) => {
        /// Available standard ANSI colors
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub enum Color {
            $( $( #[$meta] )* $color, )*
        }

        impl Color {
            /// Convert to the foreground ANSI code
            pub fn foreground_code(&self) -> &'static str {
                match self {
                    $( Color::$color => stringify!($fg), )*
                }
            }

            /// Convert to the background ANSI code
            pub fn background_code(&self) -> &'static str {
                match self {
                    $( Color::$color => stringify!($bg), )*
                }
            }
        }

        impl FromStr for Color {
            type Err = InvalidColorError;

            fn from_str(name: &str) -> Result<Self, Self::Err> {
                Ok(match name.to_ascii_lowercase().as_str() {
                    $( $names => Color::$color, )*
                    _ => return Err(InvalidColorError),
                })
            }
        }
    };
}

colors! {
    Black   30 40 ("black"),
    Red     31 41 ("red"),
    Green   32 42 ("green"),
    Yellow  33 43 ("yellow"),
    Blue    34 44 ("blue"),
    Magenta 35 45 ("magenta"),
    Cyan    36 46 ("cyan"),
    White   37 47 ("white"),
    #[default]
    Default 39 49 ("default"),

    BrightBlack   90 100 ("bright-black"),
    BrightRed     91 101 ("bright-red"),
    BrightGreen   92 102 ("bright-green"),
    BrightYellow  93 103 ("bright-yellow"),
    BrightBlue    94 104 ("bright-blue"),
    BrightMagenta 95 105 ("bright-magenta"),
    BrightCyan    96 106 ("bright-cyan"),
    BrightWhite   97 107 ("bright-white"),
}
