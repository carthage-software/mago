use std::borrow::Cow;

use crate::common::LetterCase;
use crate::common::is_case_snake_like;
use crate::common::to_case_snake_like;

/// Converts input bytes to canonical ASCII `kebab-case`.
///
/// Non-alphanumeric bytes are treated as separators. The resulting output is
/// lowercase words delimited by `-`.
#[inline]
#[must_use]
pub fn to_kebab_case<T>(input: &T) -> Cow<'_, [u8]>
where
    T: AsRef<[u8]> + ?Sized,
{
    to_case_snake_like(input.as_ref(), b'-', LetterCase::Lower, true)
}

/// Returns `true` when input is canonical ASCII `kebab-case`.
#[inline]
#[must_use]
pub fn is_kebab_case<T>(input: T) -> bool
where
    T: AsRef<[u8]>,
{
    is_case_snake_like(input.as_ref(), b'-', LetterCase::Lower, true)
}

#[cfg(test)]
mod tests {
    use super::is_kebab_case;
    use super::to_kebab_case;

    #[test]
    fn converts_examples() {
        assert_eq!(to_kebab_case("foo-bar").as_ref(), b"foo-bar");
        assert_eq!(to_kebab_case("FOO_BAR").as_ref(), b"foo-bar");
        assert_eq!(to_kebab_case("foo_bar").as_ref(), b"foo-bar");
        assert_eq!(to_kebab_case("Foo Bar").as_ref(), b"foo-bar");
        assert_eq!(to_kebab_case("Foo bar").as_ref(), b"foo-bar");
        assert_eq!(to_kebab_case("FooBar").as_ref(), b"foo-bar");
        assert_eq!(to_kebab_case("fooBar").as_ref(), b"foo-bar");
    }

    #[test]
    fn checks_examples() {
        assert!(is_kebab_case("foo-bar-string-that-is-really-really-long"));
        assert!(!is_kebab_case("FooBarIsAReallyReallyLongString"));
        assert!(!is_kebab_case("fooBarIsAReallyReallyLongString"));
        assert!(!is_kebab_case("FOO_BAR_STRING_THAT_IS_REALLY_REALLY_LONG"));
        assert!(!is_kebab_case("foo_bar_string_that_is_really_really_long"));
        assert!(!is_kebab_case("Foo bar string that is really really long"));
        assert!(!is_kebab_case("Foo Bar Is A Really Really Long String"));
    }

    #[test]
    fn supports_byte_input() {
        assert_eq!(to_kebab_case(b"foo_bar").as_ref(), b"foo-bar");
        assert!(is_kebab_case(b"foo-bar"));
    }

    #[test]
    fn borrows_when_already_canonical() {
        let input = b"foo-bar";
        assert!(matches!(to_kebab_case(input), std::borrow::Cow::Borrowed(_)));
    }
}
