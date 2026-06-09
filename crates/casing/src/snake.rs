use std::borrow::Cow;

use crate::common::LetterCase;
use crate::common::is_case_snake_like;
use crate::common::to_case_snake_like;

/// Converts input bytes to canonical ASCII `snake_case`.
///
/// This crate's snake variant keeps numeric runs attached to their adjacent
/// words when possible (`fooBar3` -> `foo_bar3`, `lower2upper` ->
/// `lower2upper`).
#[inline]
#[must_use]
pub fn to_snake_case<T>(input: &T) -> Cow<'_, [u8]>
where
    T: AsRef<[u8]> + ?Sized,
{
    to_case_snake_like(input.as_ref(), b'_', LetterCase::Lower, false)
}

/// Returns `true` when input is canonical ASCII `snake_case` for this crate's
/// numeric-friendly variant.
#[inline]
#[must_use]
pub fn is_snake_case<T>(input: T) -> bool
where
    T: AsRef<[u8]>,
{
    is_case_snake_like(input.as_ref(), b'_', LetterCase::Lower, false)
}

#[cfg(test)]
mod tests {
    use super::is_snake_case;
    use super::to_snake_case;

    #[test]
    fn converts_examples() {
        assert_eq!(to_snake_case("foo_2_bar").as_ref(), b"foo_2_bar");
        assert_eq!(to_snake_case("foo_bar").as_ref(), b"foo_bar");
        assert_eq!(to_snake_case("HTTP Foo bar").as_ref(), b"http_foo_bar");
        assert_eq!(to_snake_case("HTTPFooBar").as_ref(), b"http_foo_bar");
        assert_eq!(to_snake_case("Foo bar").as_ref(), b"foo_bar");
        assert_eq!(to_snake_case("Foo Bar").as_ref(), b"foo_bar");
        assert_eq!(to_snake_case("FooBar").as_ref(), b"foo_bar");
        assert_eq!(to_snake_case("FOO_BAR").as_ref(), b"foo_bar");
        assert_eq!(to_snake_case("fooBar").as_ref(), b"foo_bar");
        assert_eq!(to_snake_case("fooBar3").as_ref(), b"foo_bar3");
        assert_eq!(to_snake_case("lower2upper").as_ref(), b"lower2upper");
    }

    #[test]
    fn checks_examples() {
        assert!(is_snake_case("foo_2_bar"));
        assert!(is_snake_case("foo2bar"));
        assert!(is_snake_case("foo_bar"));
        assert!(is_snake_case("http_foo_bar"));
        assert!(is_snake_case("foo"));

        assert!(!is_snake_case("FooBar"));
        assert!(!is_snake_case("FooBarIsAReallyReallyLongString"));
        assert!(!is_snake_case("FooBarIsAReallyReallyLongStrings"));
        assert!(!is_snake_case("foo-bar-string-that-is-really-really-long"));
    }

    #[test]
    fn supports_byte_input() {
        assert_eq!(to_snake_case(b"fooBar").as_ref(), b"foo_bar");
        assert!(is_snake_case(b"foo_bar"));
    }

    #[test]
    fn borrows_when_already_canonical() {
        let input = b"foo_bar";
        assert!(matches!(to_snake_case(input), std::borrow::Cow::Borrowed(_)));
    }
}
