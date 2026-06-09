use std::borrow::Cow;

use crate::common::CamelLikeOptions;
use crate::common::is_case_camel_like;
use crate::common::to_case_camel_like;

const OPTIONS: CamelLikeOptions = CamelLikeOptions {
    new_word: false,
    first_word: false,
    separator: b' ',
    has_separator: false,
    inverted: false,
    concat_num: true,
};

/// Converts input bytes to canonical ASCII `camelCase`.
///
/// This function accepts any byte-backed input. If the input is already
/// canonical camel case, it returns a borrowed slice. Otherwise it returns a
/// newly allocated normalized buffer.
#[inline]
#[must_use]
pub fn to_camel_case<T>(input: &T) -> Cow<'_, [u8]>
where
    T: AsRef<[u8]> + ?Sized,
{
    to_case_camel_like(input.as_ref(), OPTIONS)
}

/// Returns `true` if input is canonical ASCII `camelCase`.
///
/// Canonicality is defined by the exact normalization performed by
/// `to_camel_case`.
#[inline]
#[must_use]
pub fn is_camel_case<T>(input: T) -> bool
where
    T: AsRef<[u8]>,
{
    is_case_camel_like(input.as_ref(), OPTIONS)
}

#[cfg(test)]
mod tests {
    use super::is_camel_case;
    use super::to_camel_case;

    #[test]
    fn converts_examples() {
        assert_eq!(to_camel_case("fooBar").as_ref(), b"fooBar");
        assert_eq!(to_camel_case("FOO_BAR").as_ref(), b"fooBar");
        assert_eq!(to_camel_case("Foo Bar").as_ref(), b"fooBar");
        assert_eq!(to_camel_case("foo_bar").as_ref(), b"fooBar");
        assert_eq!(to_camel_case("Foo bar").as_ref(), b"fooBar");
        assert_eq!(to_camel_case("foo-bar").as_ref(), b"fooBar");
        assert_eq!(to_camel_case("FooBar").as_ref(), b"fooBar");
        assert_eq!(to_camel_case("FooBar3").as_ref(), b"fooBar3");
        assert_eq!(to_camel_case("Foo-Bar").as_ref(), b"fooBar");
    }

    #[test]
    fn checks_examples() {
        assert!(is_camel_case("foo"));
        assert!(is_camel_case("fooBarIsAReallyReally3LongString"));
        assert!(is_camel_case("fooBarIsAReallyReallyLongString"));

        assert!(!is_camel_case("Foo"));
        assert!(!is_camel_case("foo-bar-string-that-is-really-really-long"));
        assert!(!is_camel_case("FooBarIsAReallyReallyLongString"));
        assert!(!is_camel_case("FOO_BAR_STRING_THAT_IS_REALLY_REALLY_LONG"));
        assert!(!is_camel_case("foo_bar_string_that_is_really_really_long"));
        assert!(!is_camel_case("Foo bar string that is really really long"));
        assert!(!is_camel_case("Foo Bar Is A Really Really Long String"));
    }

    #[test]
    fn supports_byte_input() {
        assert_eq!(to_camel_case(b"foo_bar").as_ref(), b"fooBar");
        assert!(is_camel_case(b"fooBar"));
    }

    #[test]
    fn borrows_when_already_canonical() {
        let input = b"fooBar";
        assert!(matches!(to_camel_case(input), std::borrow::Cow::Borrowed(_)));
    }
}
