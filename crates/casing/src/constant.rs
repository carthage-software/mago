use std::borrow::Cow;

use crate::common::LetterCase;
use crate::common::is_case_snake_like;
use crate::common::to_case_snake_like;

/// Converts input bytes to canonical ASCII `CONSTANT_CASE`.
///
/// Non-alphanumeric bytes are treated as separators. The resulting output is
/// uppercase words delimited by `_`.
#[inline]
#[must_use]
pub fn to_constant_case<T>(input: &T) -> Cow<'_, [u8]>
where
    T: AsRef<[u8]> + ?Sized,
{
    to_case_snake_like(input.as_ref(), b'_', LetterCase::Upper, true)
}

/// Returns `true` when input is canonical ASCII `CONSTANT_CASE`.
#[inline]
#[must_use]
pub fn is_constant_case<T>(input: T) -> bool
where
    T: AsRef<[u8]>,
{
    is_case_snake_like(input.as_ref(), b'_', LetterCase::Upper, true)
}

#[cfg(test)]
mod tests {
    use super::is_constant_case;
    use super::to_constant_case;

    #[test]
    fn converts_examples() {
        assert_eq!(to_constant_case("foo_bar").as_ref(), b"FOO_BAR");
        assert_eq!(to_constant_case("HTTP Foo bar").as_ref(), b"HTTP_FOO_BAR");
        assert_eq!(to_constant_case("Foo bar").as_ref(), b"FOO_BAR");
        assert_eq!(to_constant_case("Foo Bar").as_ref(), b"FOO_BAR");
        assert_eq!(to_constant_case("FooBar").as_ref(), b"FOO_BAR");
        assert_eq!(to_constant_case("fooBar").as_ref(), b"FOO_BAR");
        assert_eq!(to_constant_case("fooBar3").as_ref(), b"FOO_BAR_3");
    }

    #[test]
    fn checks_examples() {
        assert!(is_constant_case("FOO_BAR_STRING_THAT_IS_REALLY_REALLY_LONG"));
        assert!(is_constant_case("FOO_BAR1_STRING_THAT_IS_REALLY_REALLY_LONG"));
        assert!(is_constant_case("FOO_BAR_1_STRING_THAT_IS_REALLY_REALLY_LONG"));

        assert!(!is_constant_case("Foo bar string that is really really long"));
        assert!(!is_constant_case("foo-bar-string-that-is-really-really-long"));
        assert!(!is_constant_case("FooBarIsAReallyReallyLongString"));
        assert!(!is_constant_case("Foo Bar Is A Really Really Long String"));
        assert!(!is_constant_case("fooBarIsAReallyReallyLongString"));
    }

    #[test]
    fn supports_byte_input() {
        assert_eq!(to_constant_case(b"foo_bar").as_ref(), b"FOO_BAR");
        assert!(is_constant_case(b"FOO_BAR"));
    }

    #[test]
    fn borrows_when_already_canonical() {
        let input = b"FOO_BAR";
        assert!(matches!(to_constant_case(input), std::borrow::Cow::Borrowed(_)));
    }
}
