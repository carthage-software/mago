use std::borrow::Cow;

use mago_interner::ThreadedInterner;

use crate::enum_exists;
use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::r#enum::TEnum;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::atomic_comparator;

#[inline]
pub fn is_contained_by(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    input_class_string: &TClassLikeString,
    container_class_string: &TClassLikeString,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    let fake_container_type = match container_class_string {
        TClassLikeString::Any { .. } => {
            return true;
        }
        TClassLikeString::Literal { value } => {
            if let TClassLikeString::Literal { value: input_value } = input_class_string
                && value == input_value
            {
                return true;
            }

            if enum_exists(codebase, interner, value) {
                Cow::Owned(TAtomic::Object(TObject::Enum(TEnum::new(*value))))
            } else {
                Cow::Owned(TAtomic::Object(TObject::Named(TNamedObject::new(*value))))
            }
        }
        TClassLikeString::Generic { constraint, .. } => Cow::Borrowed(constraint.as_ref()),
        TClassLikeString::OfType { constraint, .. } => Cow::Borrowed(constraint.as_ref()),
    };

    let fake_input_type = match input_class_string {
        TClassLikeString::Any { .. } => {
            return matches!(fake_container_type.as_ref(), TAtomic::Object(TObject::Any));
        }
        TClassLikeString::Literal { value } => {
            if enum_exists(codebase, interner, value) {
                Cow::Owned(TAtomic::Object(TObject::Enum(TEnum::new(*value))))
            } else {
                Cow::Owned(TAtomic::Object(TObject::Named(TNamedObject::new(*value))))
            }
        }
        TClassLikeString::Generic { constraint, .. } => Cow::Borrowed(constraint.as_ref()),
        TClassLikeString::OfType { constraint, .. } => Cow::Borrowed(constraint.as_ref()),
    };

    atomic_comparator::is_contained_by(
        codebase,
        interner,
        fake_input_type.as_ref(),
        fake_container_type.as_ref(),
        inside_assertion,
        atomic_comparison_result,
    )
}
