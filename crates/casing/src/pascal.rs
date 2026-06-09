use std::borrow::Cow;

use crate::common::CamelLikeOptions;
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

/// Converts input bytes to canonical ASCII `PascalCase`.
///
/// The function returns a borrowed slice when the input is already canonical,
/// otherwise it returns an owned normalized buffer.
#[inline]
#[must_use]
pub fn to_pascal_case<T>(input: &T) -> Cow<'_, [u8]>
where
    T: AsRef<[u8]> + ?Sized,
{
    to_case_camel_like(input.as_ref(), OPTIONS)
}

/// Returns `true` when input is canonical ASCII `PascalCase`.
#[inline]
#[must_use]
pub fn is_pascal_case<T>(input: T) -> bool
where
    T: AsRef<[u8]>,
{
    is_case_camel_like(input.as_ref(), OPTIONS)
}

#[cfg(test)]
mod tests {
    use super::is_pascal_case;
    use super::to_pascal_case;

    #[test]
    fn converts_examples() {
        assert_eq!(to_pascal_case("fooBar").as_ref(), b"FooBar");
        assert_eq!(to_pascal_case("FOO_BAR").as_ref(), b"FooBar");
        assert_eq!(to_pascal_case("Foo Bar").as_ref(), b"FooBar");
        assert_eq!(to_pascal_case("foo_bar").as_ref(), b"FooBar");
        assert_eq!(to_pascal_case("Foo bar").as_ref(), b"FooBar");
        assert_eq!(to_pascal_case("foo-bar").as_ref(), b"FooBar");
        assert_eq!(to_pascal_case("FooBar").as_ref(), b"FooBar");
        assert_eq!(to_pascal_case("FooBar3").as_ref(), b"FooBar3");
    }

    #[test]
    fn checks_examples() {
        assert!(is_pascal_case("Foo"));
        assert!(is_pascal_case("FooBarIsAReallyReallyLongString"));
        assert!(is_pascal_case("FooBarIsAReallyReally3LongString"));

        assert!(!is_pascal_case("foo"));
        assert!(!is_pascal_case("foo-bar-string-that-is-really-really-long"));
        assert!(!is_pascal_case("FOO_BAR_STRING_THAT_IS_REALLY_REALLY_LONG"));
        assert!(!is_pascal_case("foo_bar_string_that_is_really_really_long"));
        assert!(!is_pascal_case("Foo bar string that is really really long"));
        assert!(!is_pascal_case("Foo Bar Is A Really Really Long String"));
    }

    #[test]
    fn supports_byte_input() {
        assert_eq!(to_pascal_case(b"foo_bar").as_ref(), b"FooBar");
        assert!(is_pascal_case(b"FooBar"));
    }

    #[test]
    fn borrows_when_already_canonical() {
        let input = b"FooBar";
        assert!(matches!(to_pascal_case(input), std::borrow::Cow::Borrowed(_)));
    }
}
