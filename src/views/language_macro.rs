/// This macro genrerates functions for a given enum that return language variants of a field.
/// The generatiated functions can take parameters implemented with the `std::fmt::Display` trait.
/// 
/// # Notes
/// - The first language variant is considered the default value.
/// - If a language variant is not provided for a field, the default value is used.
/// - If no language string is provided for a field, a deprecated function returning "ToDo!" is generated. The function signature stays the same.
/// - Parameter funtions return a `String` type, while non-parameter functions return a `&'static str` type.
///
/// # Syntax
/// ```rust
/// generate_language_functions! {
///     LanguageEnum: EnumName
///     field_name { lang1: "value1", lang2: "value2", ... }*
///     field_name_with_args(arg1, arg2) { lang1: "value1: {arg1}, {arg2}", lang2: "value2: {arg1}, {arg2}", ... }*
/// }
/// ```
///
/// # Parameters
/// - `LanguageEnum`: The name of the enum for which the functions are generated.
/// - `field_name`: The name of the function to be generated.
/// - `lang1`, `lang2`, ...: The language variants of the enum.
/// - `value1`, `value2`, ...: The string values corresponding to each language variant.
/// - `arg1`, `arg2`, ...: The arguments for the function if it takes any.
/// 
/// # Example
/// ```rust
/// enum Language {
///     English,
///     Spanish,
///     French,
/// }
/// 
/// generate_language_functions! {
///     LanguageEnum: Language
///     greeting { 
///         English: "Hello" 
///         Spanish: "Hola"
///         French:  "Bonjour" 
///     }
///     farewell(name) {
///         English: "Goodbye, {name}"
///         Spanish: "Adiós, {name}"
///         French:  "Au revoir, {name}"
///     }
///     calculate(a, b, c) {
///         English: "{a} + {b} = {c}"
///     }   
/// }
/// 
/// fn main() {
///     let lang = Language::English;
///     assert_eq!(lang.greeting(), "Hello");
///     assert_eq!(lang.farewell("John"), "Goodbye, John");
///     assert_eq!(lang.calculate(1, 2, 3), "1 + 2 = 3");
/// 
///     let lang = Language::Spanish;
///     assert_eq!(lang.greeting(), "Hola");
///     assert_eq!(lang.farewell("Juan"), "Adiós, Juan");
///     assert_eq!(lang.calculate(1, 2, 3), "1 + 2 = 3");
/// }
/// ```
/// # Expands to
/// 
/// ```rust	
/// #[allow(unreachable_patterns)]
/// #[allow(non_camel_case_types)]
/// impl Language {
///     pub fn greeting(&self) -> &'static str {
///         match self {
///             Language::Spanish => "Hola",
///             Language::French => "Bonjour",
///             Language::English | _ => "Hello",
///         }
///     }
///     pub fn farewell<name: std::fmt::Display>(&self, name: name) -> String {
///         match self {
///             Language::Spanish => format!("Adiós, {name}"),
///             Language::French => format!("Au revoir, {name}"),
///             Language::English | _ => format!("Goodbye, {name}"),
///         }
///     }
///     pub fn calculate<
///         a: std::fmt::Display,
///         b: std::fmt::Display,
///         c: std::fmt::Display,
///     >(&self, a: a, b: b, c: c) -> String {
///         match self {
///             Language::English | _ => format!("{a} + {b} = {c}"),
///         }
///     }
/// }
/// ```


#[macro_export]
macro_rules! generate_language_functions {
    (
        LanguageEnum: $enum_name:ident
        $($field:ident $( ( $($args:ident),+ ) )? { 
            $($lang:ident: $value:expr $(,)? )*
        })*
    ) => {
        #[allow(unreachable_patterns)]
        #[allow(non_camel_case_types)]
        impl $enum_name {
            $(
                generate_language_functions!(@field_impl $enum_name $field $( ( $($args),* ) )? { $($lang: $value,)* } );
            )*
        }
    };

    (@field_impl $enum_name:ident $field:ident { } ) => {
        #[deprecated(note = "No language string provided for this field. Defaulting to 'ToDo!'")]
        pub fn $field(&self) -> &'static str {
            "ToDo!"
        }
    };

    (@field_impl $enum_name:ident $field:ident { 
        $first_lang:ident: $first_value:expr, 
        $($lang:ident: $value:expr,)* 
    }) => {
        pub fn $field(&self) -> &'static str {
            match self {
                $( $enum_name::$lang => $value, )*
                $enum_name::$first_lang | _ => $first_value,
            }
        }
    };

    (@field_impl $enum_name:ident $field:ident ( $($args:ident),* ) { } ) => {
        #[deprecated(note = "No language string provided for this field. Defaulting to 'ToDo!'")]
        pub fn $field<$( $args: std::fmt::Display, )*>(
            &self, 
            $( $args: $args, )*
        ) -> String {
            String::from("ToDo!")
        }
    };

    (@field_impl $enum_name:ident $field:ident ( $($args:ident),* ) { 
        $first_lang:ident: $first_value:expr, 
        $($lang:ident: $value:expr,)* 
    } ) => {
        pub fn $field<$( $args: std::fmt::Display, )*>(
            &self,
            $( $args: $args, )*
        ) -> String {
            match self {
                $( $enum_name::$lang => format!($value), )*
                $enum_name::$first_lang | _ => format!($first_value),
            }
        }
    };
}