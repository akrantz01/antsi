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
