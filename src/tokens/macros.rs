/// Remove the error generic from the specified function
macro_rules! make_error_concrete {
    ($name:ident -> $ret:ty) => {
        fn $name(input: &str) -> ::nom::IResult<&str, $ret, ::nom::error::Error<&str>> {
            super::super::$name(input)
        }
    };
}

/// Create a sequence of tests for success for a parser
macro_rules! simple_tests {
    (
        for $function:ident;
        $( $name:ident: $input:literal => $output:expr ),* $(,)?
    ) => {
        $(
            #[test]
            fn $name () {
                assert_eq!($function($input), Ok(("", From::from($output))));
            }
        )*
    };
}

/// Create a new style
macro_rules! style {
    () => {
        $crate::tokens::Style::default()
    };
    (@internal $style:expr; fg: $color:ident ; $( $rest:tt )* ) => {{
        $style.foreground = Some($crate::styles::Color::$color);
        style!(@internal $style; $( $rest ) *)
    }};
    (@internal $style:expr; bg: $color:ident ; $( $rest:tt )* ) => {{
        $style.background = Some($crate::styles::Color::$color);
        style!(@internal $style; $( $rest ) *)
    }};
    (@internal $style:expr; deco: $( $decoration:ident ),+ ; $( $rest:tt )* ) => {{
        $style.decoration = Some(vec![ $( $crate::styles::Decoration::$decoration, )+ ]);
        style!(@internal $style; $( $rest ) *)
    }};
    (@internal $style:expr; ) => {
        $style
    };
    ( $( $rest:tt )* ) => {{
        let mut style = $crate::tokens::Style::default();
        style!(@internal style; $( $rest )*)
    }};
}
