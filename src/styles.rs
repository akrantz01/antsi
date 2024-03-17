macro_rules! colors {
    (
        $( $color:ident $fg:literal $bg:literal ),* $(,)?
    ) => {
        /// Available standard ANSI colors
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum Color {
            $( $color, )*
        }

        impl Color {
            /// Convert to the foreground ANSI code
            pub fn foreground_code(&self) -> u8 {
                match self {
                    $( Color::$color => $fg, )*
                }
            }

            /// Convert to the background ANSI code
            pub fn background_code(&self) -> u8 {
                match self {
                    $( Color::$color => $bg, )*
                }
            }
        }
    };
}

macro_rules! decorations {
    (
        $( $decoration:ident $apply:literal $remove:literal ),* $(,)?
    ) => {
        /// Available standard ANSI text decorations
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum Decoration {
            $( $decoration, )*
        }

        impl Decoration {
            /// Convert to the ANSI code for applying the styling
            pub fn apply_code(&self) -> u8 {
                match self {
                    $( Decoration::$decoration => $apply, )*
                }
            }

            /// Convert to the ANSI code for removing the styling
            pub fn remove_code(&self) -> u8 {
                match self {
                    $( Decoration::$decoration => $remove, )*
                }
            }
        }
    };
}

colors! {
    Black   30 40,
    Red     31 41,
    Green   32 42,
    Yellow  33 43,
    Blue    34 44,
    Magenta 35 45,
    Cyan    36 46,
    White   37 47,
    Default 39 49,

    BrightBlack   90 100,
    BrightRed     91 101,
    BrightGreen   92 102,
    BrightYellow  93 103,
    BrightBlue    94 104,
    BrightMagenta 95 105,
    BrightCyan    96 106,
    BrightWhite   97 107,
}

decorations! {
    Bold          1 22,
    Dim           2 22,
    Italic        3 23,
    Underline     4 24,
    SlowBlink     5 25,
    FastBlink     6 25,
    Invert        7 27,
    Hide          8 28,
    StrikeThrough 9 29,
}
