use std::borrow::Cow;

use crate::common::CamelLikeOptions;
use crate::common::is_ascii_digit;
use crate::common::is_ascii_lowercase;
use crate::common::is_ascii_uppercase;
use crate::common::is_case_camel_like;
use crate::common::to_case_camel_like;

const OPTIONS: CamelLikeOptions = CamelLikeOptions {
    new_word: true,
    first_word: false,
    separator: b' ',
    has_separator: false,
    inverted: false,
    concat_num: true,
};

/// Converts input bytes to this crate's ASCII `ClassCase` variant.
///
/// This variant intentionally does not singularize words and preserves leading
/// acronym-like prefixes such as `HTTP2` and `UT8`.
#[inline]
#[must_use]
pub fn to_class_case<T>(input: &T) -> Cow<'_, [u8]>
where
    T: AsRef<[u8]> + ?Sized,
{
    let input = input.as_ref();

    if is_class_case(input) { Cow::Borrowed(input) } else { Cow::Owned(convert_class_case(input)) }
}

/// Returns `true` when input is canonical for this crate's `ClassCase`.
#[inline]
#[must_use]
pub fn is_class_case<T>(input: T) -> bool
where
    T: AsRef<[u8]>,
{
    let input = input.as_ref();

    if input.is_empty() {
        return false;
    }

    if !is_ascii_uppercase(input[0]) {
        return false;
    }

    let mut cursor = 0;

    while cursor < input.len() {
        let segment = &input[cursor..];
        let prefix_len = class_prefix_len(segment);

        if prefix_len == 0 {
            return is_case_camel_like(segment, OPTIONS);
        }

        cursor += prefix_len;
    }

    true
}

#[inline]
fn class_prefix_len(input: &[u8]) -> usize {
    let mut index = 0;
    let mut prefix_length = 0;

    while index < input.len() {
        let byte = input[index];

        if is_ascii_uppercase(byte) || is_ascii_digit(byte) {
            prefix_length += 1;
            index += 1;
            continue;
        }

        if is_ascii_lowercase(byte) && prefix_length > 0 {
            prefix_length += 1;
            index += 1;

            while index < input.len() {
                let tail = input[index];
                if is_ascii_lowercase(tail) || is_ascii_digit(tail) {
                    prefix_length += 1;
                    index += 1;
                } else {
                    break;
                }
            }

            break;
        }

        break;
    }

    prefix_length
}

#[inline]
fn convert_class_case(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len() * 2);
    let mut cursor = 0;

    while cursor < input.len() {
        let segment = &input[cursor..];
        let prefix_len = class_prefix_len(segment);

        if prefix_len == 0 {
            output.extend_from_slice(to_case_camel_like(segment, OPTIONS).as_ref());
            break;
        }

        output.extend_from_slice(&segment[..prefix_len]);
        cursor += prefix_len;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::is_class_case;
    use super::to_class_case;

    #[test]
    fn converts_examples() {
        assert_eq!(to_class_case("UInt").as_ref(), b"UInt");
        assert_eq!(to_class_case("Uint").as_ref(), b"Uint");
        assert_eq!(to_class_case("Http2Client").as_ref(), b"Http2Client");
        assert_eq!(to_class_case("Fl3xxSomething").as_ref(), b"Fl3xxSomething");
        assert_eq!(to_class_case("IsUT8Test").as_ref(), b"IsUT8Test");
        assert_eq!(to_class_case("HTTP2Client").as_ref(), b"HTTP2Client");
        assert_eq!(to_class_case("FooBar").as_ref(), b"FooBar");
        assert_eq!(to_class_case("FooBars").as_ref(), b"FooBars");
        assert_eq!(to_class_case("foo_bars").as_ref(), b"FooBars");
        assert_eq!(to_class_case("Foo Bar").as_ref(), b"FooBar");
        assert_eq!(to_class_case("foo-bar").as_ref(), b"FooBar");
        assert_eq!(to_class_case("fooBar").as_ref(), b"FooBar");
        assert_eq!(to_class_case("Foo_Bar").as_ref(), b"FooBar");
        assert_eq!(to_class_case("Foo bar").as_ref(), b"FooBar");
    }

    #[test]
    fn checks_examples() {
        assert!(is_class_case("Foo"));
        assert!(is_class_case("FooBarIsAReallyReallyLongString"));
        assert!(is_class_case("FooBarIsAReallyReallyLongStrings"));
        assert!(is_class_case("UInt"));
        assert!(is_class_case("Uint"));
        assert!(is_class_case("Http2Client"));
        assert!(is_class_case("Fl3xxSomething"));
        assert!(is_class_case("IsUT8Test"));
        assert!(is_class_case("HTTP2Client"));

        assert!(!is_class_case("foo"));
        assert!(!is_class_case("foo-bar-string-that-is-really-really-long"));
        assert!(!is_class_case("foo_bar_is_a_really_really_long_strings"));
        assert!(!is_class_case("fooBarIsAReallyReallyLongString"));
        assert!(!is_class_case("FOO_BAR_STRING_THAT_IS_REALLY_REALLY_LONG"));
        assert!(!is_class_case("foo_bar_string_that_is_really_really_long"));
        assert!(!is_class_case("Foo bar string that is really really long"));
        assert!(!is_class_case("Foo Bar Is A Really Really Long String"));
    }

    #[test]
    fn supports_byte_input() {
        assert_eq!(to_class_case(b"foo_bar").as_ref(), b"FooBar");
        assert!(is_class_case(b"FooBar"));
    }

    #[test]
    fn borrows_when_already_canonical() {
        let input = b"HTTP2Client";
        assert!(matches!(to_class_case(input), std::borrow::Cow::Borrowed(_)));
    }
}
