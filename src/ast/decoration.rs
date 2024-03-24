use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

/// The provided [`Decoration`] name was invalid
#[derive(Clone, Copy, Debug)]
pub struct InvalidDecorationError;

impl std::error::Error for InvalidDecorationError {}

impl Display for InvalidDecorationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid decoration name")
    }
}

macro_rules! decorations {
    (
        $( $decoration:ident $apply:literal $remove:literal ( $names:pat ) ),* $(,)?
    ) => {
        /// Available standard ANSI text decorations
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum Decoration {
            $( $decoration, )*
        }

        impl Decoration {
            /// Convert to the ANSI code for applying the styling
            pub fn apply_code(&self) -> &'static str {
                match self {
                    $( Decoration::$decoration => stringify!($apply), )*
                }
            }

            /// Convert to the ANSI code for removing the styling
            pub fn remove_code(&self) -> &'static str {
                match self {
                    $( Decoration::$decoration => stringify!($remove), )*
                }
            }
        }

        impl FromStr for Decoration {
            type Err = InvalidDecorationError;

            fn from_str(name: &str) -> Result<Self, Self::Err> {
                Ok(match name.to_ascii_lowercase().as_str() {
                    $( $names => Decoration::$decoration, )*
                    _ => return Err(InvalidDecorationError),
                })
            }
        }
    };
}

decorations! {
    Bold          1 22 ("bold"),
    Dim           2 22 ("dim" | "faint"),
    Italic        3 23 ("italic"),
    Underline     4 24 ("underline"),
    SlowBlink     5 25 ("slow-blink" | "blink-slow"),
    FastBlink     6 25 ("fast-blink" | "blink-fast"),
    Invert        7 27 ("invert" | "reverse"),
    Hide          8 28 ("hide" | "conceal"),
    StrikeThrough 9 29 ("strikethrough" | "strike-through"),
}
