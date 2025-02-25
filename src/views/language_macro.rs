#[macro_export]
macro_rules! generate_language_functions {
    (
        $(
            $field:ident $( ( $($args:ident),+ ) )? { 
                $($lang:ident: $value:expr $(,)? )*
            }
        )*
    ) => {
        #[allow(dead_code)]
        #[allow(unreachable_patterns)]
        #[allow(non_camel_case_types)]
        impl Language {
            $(
                generate_language_functions!(@field_impl $field $( ( $($args),* ) )? { $($lang: $value,)* } );
            )*
        }
    };

    (@field_impl $field:ident { } ) => {
        #[deprecated(note = "No language string provided for this field. Defaulting to 'ToDo!'")]
        pub fn $field(&self) -> &'static str {
            "ToDo!"
        }
    };

    (@field_impl $field:ident { $first_lang:ident: $first_value:expr, $($lang:ident: $value:expr,)* } ) => {
        pub fn $field(&self) -> &'static str {
            match self {
                $( Language::$lang => $value, )*
                Language::$first_lang | _ => $first_value,
            }
        }
    };

    (@field_impl $field:ident ( $($args:ident),* ) { } ) => {
        #[deprecated(note = "No language string provided for this field. Defaulting to 'ToDo!'")]
        pub fn $field<$( $args: std::fmt::Display, )*>(
            &self, 
            $( $args: $args, )*
        ) -> String {
            String::from("ToDo!")
        }
    };

    (@field_impl $field:ident ( $($args:ident),* ) { $first_lang:ident: $first_value:expr, $($lang:ident: $value:expr,)* } ) => {
        pub fn $field<$( $args: std::fmt::Display, )*>(
            &self,
            $( $args: $args, )*
        ) -> String {
            match self {
                $( Language::$lang => format!($value), )*
                Language::$first_lang | _ => format!($first_value),
            }
        }
    };
}