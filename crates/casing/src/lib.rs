pub use cruet::case::camel::is_camel_case;
pub use cruet::case::camel::to_camel_case;
pub use cruet::case::kebab::is_kebab_case;
pub use cruet::case::kebab::to_kebab_case;
pub use cruet::case::pascal::is_pascal_case;
pub use cruet::case::pascal::to_pascal_case;
pub use cruet::case::screaming_snake::is_screaming_snake_case as is_constant_case;
pub use cruet::case::screaming_snake::to_screaming_snake_case as to_constant_case;
pub use cruet::case::sentence::is_sentence_case;
pub use cruet::case::sentence::to_sentence_case;
pub use cruet::case::table::is_table_case;
pub use cruet::case::table::to_table_case;
pub use cruet::case::title::is_title_case;
pub use cruet::case::title::to_title_case;
pub use cruet::case::train::is_train_case;
pub use cruet::case::train::to_train_case;

/// Determines if a `&str` is `ClassCase` `bool`
///
/// Unlike `cruet::case::is_class_case`, this function does not
/// require the string to be in singular form.
///
/// ```
/// use mago_casing::is_class_case;
///
/// assert!(is_class_case("Foo"));
/// assert!(is_class_case("FooBarIsAReallyReallyLongString"));
/// assert!(is_class_case("FooBarIsAReallyReallyLongStrings"));
/// assert!(is_class_case("UInt"));
/// assert!(is_class_case("Uint"));
/// assert!(is_class_case("Http2Client"));
/// assert!(is_class_case("Fl3xxSomething"));
/// assert!(is_class_case("IsUT8Test"));
/// assert!(is_class_case("HTTP2Client"));
///
/// assert!(!is_class_case("foo"));
/// assert!(!is_class_case("foo-bar-string-that-is-really-really-long"));
/// assert!(!is_class_case("foo_bar_is_a_really_really_long_strings"));
/// assert!(!is_class_case("fooBarIsAReallyReallyLongString"));
/// assert!(!is_class_case("FOO_BAR_STRING_THAT_IS_REALLY_REALLY_LONG"));
/// assert!(!is_class_case("foo_bar_string_that_is_really_really_long"));
/// assert!(!is_class_case("Foo bar string that is really really long"));
/// assert!(!is_class_case("Foo Bar Is A Really Really Long String"));
/// ```
pub fn is_class_case(test_string: &str) -> bool {
    to_class_case(test_string) == test_string
}

/// Converts a `&str` to `ClassCase` `String`
///
/// Unlike `cruet::case::to_class_case`, this function does not
/// convert the string to singular form.
///
/// ```
/// use mago_casing::to_class_case;
///
/// assert_eq!(to_class_case("UInt"), "UInt");
/// assert_eq!(to_class_case("Uint"), "Uint");
/// assert_eq!(to_class_case("Http2Client"), "Http2Client");
/// assert_eq!(to_class_case("Fl3xxSomething"), "Fl3xxSomething");
/// assert_eq!(to_class_case("IsUT8Test"), "IsUT8Test");
/// assert_eq!(to_class_case("HTTP2Client"), "HTTP2Client");
/// assert_eq!(to_class_case("FooBar"), "FooBar");
/// assert_eq!(to_class_case("FooBars"), "FooBars");
/// assert_eq!(to_class_case("foo_bars"), "FooBars");
/// assert_eq!(to_class_case("Foo Bar"), "FooBar");
/// assert_eq!(to_class_case("foo-bar"), "FooBar");
/// assert_eq!(to_class_case("fooBar"), "FooBar");
/// assert_eq!(to_class_case("Foo_Bar"), "FooBar");
/// assert_eq!(to_class_case("Foo bar"), "FooBar");
/// ```
pub fn to_class_case(non_class_case_string: &str) -> String {
    // grab the prefix, which is the first N - 1 uppercase characters, leaving only one uppercase
    // character at the beginning of the string
    let mut characters = non_class_case_string.chars();
    let mut prefix_length = 0;
    loop {
        let Some(character) = characters.next() else {
            break;
        };

        if character.is_uppercase() {
            prefix_length += 1;
            continue;
        }

        if character.is_numeric() {
            prefix_length += 1;
            continue;
        }

        if character.is_lowercase() && prefix_length > 0 {
            prefix_length += 1;

            loop {
                let Some(character) = characters.next() else {
                    break;
                };

                if character.is_lowercase() || character.is_numeric() {
                    prefix_length += 1;
                } else {
                    break;
                }
            }

            break;
        }

        break;
    }

    let prefix = &non_class_case_string[..prefix_length];
    let remaining = &non_class_case_string[prefix_length..];
    if remaining.is_empty() {
        return prefix.to_string();
    }

    if prefix.is_empty() {
        return cruet::case::to_case_camel_like(
            non_class_case_string,
            cruet::case::CamelOptions {
                new_word: true,
                last_char: ' ',
                first_word: false,
                injectable_char: ' ',
                has_seperator: false,
                inverted: false,
                concat_num: true,
            },
        );
    }

    let mut class_name = crate::to_class_case(remaining);
    class_name.insert_str(0, prefix);

    class_name
}

/// Determines if a `&str` is `snake_case` `bool`
///
/// Unlike `cruet::case::is_snake_case`, this function allows for
/// numbers to be included in the string without separating them.
///
/// ```
/// use mago_casing::is_snake_case;
///
/// assert!(is_snake_case("foo_2_bar"));
/// assert!(is_snake_case("foo2bar"));
/// assert!(is_snake_case("foo_bar"));
/// assert!(is_snake_case("http_foo_bar"));
/// assert!(is_snake_case("http_foo_bar"));
/// assert!(is_snake_case("foo_bar"));
/// assert!(is_snake_case("foo"));
/// assert!(!is_snake_case("FooBar"));
/// assert!(!is_snake_case("FooBarIsAReallyReallyLongString"));
/// assert!(!is_snake_case("FooBarIsAReallyReallyLongStrings"));
/// assert!(!is_snake_case("foo-bar-string-that-is-really-really-long"));
/// ```
pub fn is_snake_case(test_string: &str) -> bool {
    test_string == to_snake_case(test_string)
}

/// Converts a `&str` to `snake_case` `String`
///
/// Unlike `cruet::case::to_snake_case`, this function allows for
/// numbers to be included in the string without separating them.
///
/// ```
/// use mago_casing::to_snake_case;
///
/// assert_eq!(to_snake_case("foo_2_bar"),  "foo_2_bar");
/// assert_eq!(to_snake_case("foo_bar"),  "foo_bar");
/// assert_eq!(to_snake_case("HTTP Foo bar"),  "http_foo_bar");
/// assert_eq!(to_snake_case("HTTPFooBar"),  "http_foo_bar");
/// assert_eq!(to_snake_case("Foo bar"),  "foo_bar");
/// assert_eq!(to_snake_case("Foo Bar"),  "foo_bar");
/// assert_eq!(to_snake_case("FooBar"),  "foo_bar");
/// assert_eq!(to_snake_case("FOO_BAR"),  "foo_bar");
/// assert_eq!(to_snake_case("fooBar"),  "foo_bar");
/// assert_eq!(to_snake_case("fooBar3"),  "foo_bar3");
/// assert_eq!(to_snake_case("lower2upper"),  "lower2upper");
/// ```
pub fn to_snake_case(non_snake_case_string: &str) -> String {
    let mut first_character: bool = true;
    let mut last_separator: bool = true;
    let mut result: String = String::with_capacity(non_snake_case_string.len() * 2);

    for char_with_index in non_snake_case_string.trim_end_matches(|c: char| !c.is_alphanumeric()).char_indices() {
        if !char_with_index.1.is_alphanumeric() {
            if !first_character && !last_separator {
                first_character = true;
                last_separator = true;
                result.push('_');
            }
        } else {
            first_character = false;
            if !last_separator
                && !first_character
                && char_with_index.1.is_uppercase()
                && (non_snake_case_string.chars().nth(char_with_index.0 + 1).unwrap_or('A').is_lowercase()
                    || non_snake_case_string.chars().nth(char_with_index.0 - 1).unwrap_or('A').is_lowercase())
            {
                last_separator = true;
                result.push('_');
            } else {
                last_separator = false;
            }

            result.push(char_with_index.1.to_ascii_lowercase());
        }
    }
    result
}
