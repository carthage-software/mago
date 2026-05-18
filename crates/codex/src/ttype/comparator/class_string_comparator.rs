use std::borrow::Cow;

use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::r#enum::TEnum;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::atomic::scalar::string::TStringLiteral;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::atomic_comparator;

#[inline]
pub fn is_contained_by(
    codebase: &CodebaseMetadata,
    input_scalar: &TScalar,
    container_class_string: &TClassLikeString,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    let fake_container_type = match container_class_string {
        TClassLikeString::Any { .. } => {
            return true;
        }
        TClassLikeString::Literal { value } => {
            if let Some(str_value) = input_scalar.get_known_literal_string_value()
                && is_valid_class_string(str_value)
                && str_value.eq_ignore_ascii_case(value.as_bytes())
            {
                return true;
            }

            if let Some(literal_class_string) = input_scalar.get_literal_class_string_value()
                && literal_class_string.as_bytes().eq_ignore_ascii_case(value.as_bytes())
            {
                return true;
            }

            if codebase.enum_exists(value.as_bytes()) {
                Cow::Owned(TAtomic::Object(TObject::Enum(TEnum::new(*value))))
            } else {
                Cow::Owned(TAtomic::Object(TObject::Named(TNamedObject::new(*value))))
            }
        }
        TClassLikeString::Generic { constraint, .. } => Cow::Borrowed(constraint.as_ref()),
        TClassLikeString::OfType { constraint, .. } => Cow::Borrowed(constraint.as_ref()),
    };

    let fake_input_type = match input_scalar {
        TScalar::String(TString { literal: Some(TStringLiteral::Value(string_value)), .. }) => {
            if !is_valid_class_string(string_value.as_bytes()) {
                return false;
            }

            if codebase.enum_exists(string_value.as_bytes()) {
                Cow::Owned(TAtomic::Object(TObject::Enum(TEnum::new(*string_value))))
            } else {
                Cow::Owned(TAtomic::Object(TObject::Named(TNamedObject::new(*string_value))))
            }
        }
        TScalar::ClassLikeString(input_class_string) => match input_class_string {
            TClassLikeString::Any { .. } => {
                return matches!(fake_container_type.as_ref(), TAtomic::Object(TObject::Any));
            }
            TClassLikeString::Literal { value } => {
                if codebase.enum_exists(value.as_bytes()) {
                    Cow::Owned(TAtomic::Object(TObject::Enum(TEnum::new(*value))))
                } else {
                    Cow::Owned(TAtomic::Object(TObject::Named(TNamedObject::new(*value))))
                }
            }
            TClassLikeString::Generic { constraint, .. } => Cow::Borrowed(constraint.as_ref()),
            TClassLikeString::OfType { constraint, .. } => Cow::Borrowed(constraint.as_ref()),
        },
        _ => {
            return false;
        }
    };

    atomic_comparator::is_contained_by(
        codebase,
        fake_input_type.as_ref(),
        fake_container_type.as_ref(),
        inside_assertion,
        atomic_comparison_result,
    )
}

fn is_valid_class_string(bytes: &[u8]) -> bool {
    let len = bytes.len();

    if len == 0 || bytes[len - 1] == b'\\' {
        return false;
    }

    let mut i = usize::from(bytes[0] == b'\\');
    if i >= len {
        return false;
    }

    let mut part_start = true;

    while i < len {
        let b = bytes[i];

        if b == b'\\' {
            if part_start {
                return false; // empty part
            }

            part_start = true;
        } else if part_start {
            if !(b.is_ascii_alphabetic() || b == b'_') {
                return false;
            }

            part_start = false;
        } else if !(b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80) {
            return false;
        }

        i += 1;
    }

    !part_start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_class_string() {
        assert!(is_valid_class_string(b"A"));
        assert!(is_valid_class_string(b"_A"));
        assert!(is_valid_class_string(b"A1"));
        assert!(is_valid_class_string(b"A\\B"));
        assert!(is_valid_class_string(b"\\A\\B"));
        assert!(is_valid_class_string("café".as_bytes()));
    }

    #[test]
    fn test_invalid_class_string() {
        assert!(!is_valid_class_string(b""));
        assert!(!is_valid_class_string(b"1A"));
        assert!(!is_valid_class_string(b"A-B"));
        assert!(!is_valid_class_string(b"A\\"));
        assert!(!is_valid_class_string(b"\\"));
        assert!(!is_valid_class_string(b"A\\\\B"));
        assert!(!is_valid_class_string(b"A B"));
    }
}
