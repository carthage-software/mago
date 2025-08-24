use std::borrow::Cow;

use mago_atom::atom;

use crate::enum_exists;
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
                && str_value.eq_ignore_ascii_case(value)
            {
                return true;
            }

            if let Some(literal_class_string) = input_scalar.get_literal_class_string_value()
                && literal_class_string.eq_ignore_ascii_case(value)
            {
                return true;
            }

            if enum_exists(codebase, value) {
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
            if !is_valid_class_string(string_value) {
                return false;
            }

            if enum_exists(codebase, string_value) {
                Cow::Owned(TAtomic::Object(TObject::Enum(TEnum::new(atom(string_value)))))
            } else {
                Cow::Owned(TAtomic::Object(TObject::Named(TNamedObject::new(atom(string_value)))))
            }
        }
        TScalar::ClassLikeString(input_class_string) => match input_class_string {
            TClassLikeString::Any { .. } => {
                return matches!(fake_container_type.as_ref(), TAtomic::Object(TObject::Any));
            }
            TClassLikeString::Literal { value } => {
                if enum_exists(codebase, value) {
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

fn is_valid_class_string(str: &str) -> bool {
    let bytes = str.as_bytes();
    let len = bytes.len();

    if len == 0 || bytes[len - 1] == b'\\' {
        return false;
    }

    let mut i = if bytes[0] == b'\\' { 1 } else { 0 };
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
        assert!(is_valid_class_string("A"));
        assert!(is_valid_class_string("_A"));
        assert!(is_valid_class_string("A1"));
        assert!(is_valid_class_string("A\\B"));
        assert!(is_valid_class_string("\\A\\B"));
        assert!(is_valid_class_string("café"));
    }

    #[test]
    fn test_invalid_class_string() {
        assert!(!is_valid_class_string(""));
        assert!(!is_valid_class_string("1A"));
        assert!(!is_valid_class_string("A-B"));
        assert!(!is_valid_class_string("A\\"));
        assert!(!is_valid_class_string("\\"));
        assert!(!is_valid_class_string("A\\\\B"));
        assert!(!is_valid_class_string("A B"));
    }
}
