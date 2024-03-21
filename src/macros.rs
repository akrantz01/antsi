/// Create a new style
macro_rules! style {
    () => {
        $crate::styles::Style::default()
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
        let mut style = $crate::styles::Style::default();
        style!(@internal style; $( $rest )*)
    }};
}
