/// Create a new set
macro_rules! set {
    ( $( $value:expr ),* $(,)? ) => {{
        const CAP: usize = <[()]>::len(&[ $( { stringify!($value); } ),* ]);
        let mut set = ::indexmap::IndexSet::with_capacity(CAP);
        $( set.insert($value); )+
        set
    }}
}

/// Create a new style
macro_rules! style {
    () => {
        $crate::ast::Style::default()
    };
    (@internal $style:expr; fg: $color:ident ; $( $rest:tt )* ) => {{
        $style.foreground = Some($crate::ast::Color::$color);
        style!(@internal $style; $( $rest ) *)
    }};
    (@internal $style:expr; bg: $color:ident ; $( $rest:tt )* ) => {{
        $style.background = Some($crate::ast::Color::$color);
        style!(@internal $style; $( $rest ) *)
    }};
    (@internal $style:expr; deco: $( $decoration:ident ),+ ; $( $rest:tt )* ) => {{
        $style.decoration = Some(set!{ $( $crate::ast::Decoration::$decoration, )+ });
        style!(@internal $style; $( $rest ) *)
    }};
    (@internal $style:expr; ) => {
        $style
    };
    ( $( $rest:tt )* ) => {{
        let mut style = $crate::ast::Style::default();
        style!(@internal style; $( $rest )*)
    }};
}

/// Create a snapshot for testing parsing
macro_rules! assert_parse_snapshot {
    ($parser:ident; $source:literal) => {
        insta::with_settings!({
            description => $source,
            omit_expression => true,
        }, {
            let mut parser = $crate::parser::Parser::new($source);
            insta::assert_debug_snapshot!($parser(&mut parser));
        });
    };
    (|$var:ident| $parser:expr; $source:literal) => {
        insta::with_settings!({
            description => $source,
            omit_expression => true,
        }, {
            let mut parser = $crate::parser::Parser::new($source);
            let f = |$var| $parser;
            insta::assert_debug_snapshot!(f(&mut parser));
        });
    };
    ($source:expr, $expression:expr) => {
        insta::with_settings!({
            description => $source,
            omit_expression => true,
        }, {
            let parser = $crate::parser::Parser::new($source);
            insta::assert_debug_snapshot!($expression);
        });
    };
}

/// Create a new [`text_size::TextRange`] for tests
macro_rules! span {
    ($from:literal .. $to:literal) => {
        ::text_size::TextRange::new($from.into(), $to.into())
    };
}
