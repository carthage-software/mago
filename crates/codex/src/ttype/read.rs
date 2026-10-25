use std::borrow::Cow;

use crate::metadata::CodebaseMetadata;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::combiner;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::atomic_comparator;
use crate::ttype::get_never;
use crate::ttype::intersect_atomic_types;
use crate::ttype::union::TUnion;

enum Intersection {
    Type(TAtomic),
    Empty,
    Unresolved,
}

#[must_use]
pub fn intersect_read_types(left: &TUnion, right: &TUnion, codebase: &CodebaseMetadata) -> TUnion {
    if left == right || right.is_vanilla_mixed() {
        return left.clone();
    }

    if left.is_vanilla_mixed() {
        return right.clone();
    }

    let mut types = Vec::new();
    let left_types = expand_scalars(left);
    let right_types = expand_scalars(right);
    for left in left_types.iter() {
        let start = types.len();
        for right in right_types.iter() {
            match intersect_atomic(left, right, codebase) {
                Intersection::Type(ty) => types.push(ty),
                Intersection::Empty => {}
                Intersection::Unresolved => {
                    types.truncate(start);
                    types.push(left.clone());
                    break;
                }
            }
        }
    }

    if types.is_empty() {
        return get_never();
    }

    TUnion::from_vec(combiner::combine(types, codebase, combiner::CombinerOptions::default()))
}

fn expand_scalars(ty: &TUnion) -> Cow<'_, [TAtomic]> {
    if !ty.types.iter().any(|ty| matches!(ty, TAtomic::Scalar(TScalar::Generic | TScalar::Numeric | TScalar::ArrayKey)))
    {
        return Cow::Borrowed(&ty.types);
    }

    let mut types = Vec::new();
    for atomic in ty.types.iter() {
        match atomic {
            TAtomic::Scalar(TScalar::Generic) => types.extend([
                TAtomic::Scalar(TScalar::bool()),
                TAtomic::Scalar(TScalar::int()),
                TAtomic::Scalar(TScalar::float()),
                TAtomic::Scalar(TScalar::string()),
            ]),
            TAtomic::Scalar(TScalar::Numeric) => types.extend([
                TAtomic::Scalar(TScalar::int()),
                TAtomic::Scalar(TScalar::float()),
                TAtomic::Scalar(TScalar::numeric_string()),
            ]),
            TAtomic::Scalar(TScalar::ArrayKey) => {
                types.extend([TAtomic::Scalar(TScalar::int()), TAtomic::Scalar(TScalar::string())]);
            }
            _ => types.push(atomic.clone()),
        }
    }

    Cow::Owned(types)
}

fn intersect_atomic(left: &TAtomic, right: &TAtomic, codebase: &CodebaseMetadata) -> Intersection {
    match (left, right) {
        (TAtomic::Scalar(TScalar::String(left)), TAtomic::Scalar(TScalar::String(right))) => {
            return left
                .intersect(right)
                .map_or(Intersection::Empty, |ty| Intersection::Type(TAtomic::Scalar(TScalar::String(ty))));
        }
        (TAtomic::Scalar(TScalar::Integer(_)), TAtomic::Scalar(TScalar::Integer(_))) => {
            return intersect_atomic_types(left, right, codebase, &mut false)
                .map_or(Intersection::Empty, Intersection::Type);
        }
        _ => {}
    }

    if atomic_comparator::is_contained_by(codebase, left, right, true, &mut ComparisonResult::new()) {
        return Intersection::Type(left.clone());
    }

    if atomic_comparator::is_contained_by(codebase, right, left, true, &mut ComparisonResult::new()) {
        return Intersection::Type(right.clone());
    }

    match (left, right) {
        (TAtomic::Never, _) | (_, TAtomic::Never) => Intersection::Empty,
        (TAtomic::Object(TObject::Named(left_object)), TAtomic::Object(TObject::Named(right_object))) => {
            let Some(left_class) = codebase.get_class_like_by_name(left_object.name) else {
                return Intersection::Unresolved;
            };
            let Some(right_class) = codebase.get_class_like_by_name(right_object.name) else {
                return Intersection::Unresolved;
            };

            if left_class.kind.is_interface()
                || right_class.kind.is_interface()
                || left_class.kind.is_trait()
                || right_class.kind.is_trait()
            {
                let mut intersection = left.clone();
                intersection.add_intersection_type(right.clone());
                Intersection::Type(intersection)
            } else {
                Intersection::Unresolved
            }
        }
        (TAtomic::Object(TObject::Enum(_)), TAtomic::Object(TObject::Enum(_))) => Intersection::Empty,
        (
            TAtomic::Scalar(TScalar::Bool(_) | TScalar::Integer(_) | TScalar::Float(_) | TScalar::String(_)),
            TAtomic::Scalar(TScalar::Bool(_) | TScalar::Integer(_) | TScalar::Float(_) | TScalar::String(_)),
        )
        | (TAtomic::Null, TAtomic::Scalar(_) | TAtomic::Array(_) | TAtomic::Object(_))
        | (TAtomic::Scalar(_) | TAtomic::Array(_) | TAtomic::Object(_), TAtomic::Null)
        | (TAtomic::Array(_) | TAtomic::Object(_), TAtomic::Scalar(_))
        | (TAtomic::Scalar(_), TAtomic::Array(_) | TAtomic::Object(_))
        | (TAtomic::Array(_), TAtomic::Object(_))
        | (TAtomic::Object(_), TAtomic::Array(_)) => Intersection::Empty,
        _ => Intersection::Unresolved,
    }
}
