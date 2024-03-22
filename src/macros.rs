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
        $style.decoration = Some(set!{ $( $crate::styles::Decoration::$decoration, )+ });
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
