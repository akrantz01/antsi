mod color;
mod decoration;
mod style;
mod token;

pub use color::{Color, InvalidColorError};
pub use decoration::{Decoration, InvalidDecorationError};
pub use style::{CurrentStyle, Style};
pub use token::{Token, Tokens};
