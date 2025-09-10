use std::borrow::Cow;
use std::hash::Hash;
use std::hash::Hasher;

use derivative::Derivative;
use mago_atom::concat_atom;
use mago_atom::empty_atom;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;

use crate::metadata::CodebaseMetadata;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::symbol::Symbols;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::mixed::truthiness::TMixedTruthiness;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::populate_atomic_type;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::bool::TBool;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::atomic::scalar::string::TStringLiteral;
use crate::ttype::get_arraykey;
use crate::ttype::get_int;
use crate::ttype::get_mixed;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Derivative, PartialOrd, Ord)]
pub struct TUnion {
    pub types: Cow<'static, [TAtomic]>,
    pub had_template: bool,
    pub by_reference: bool,
    pub reference_free: bool,
    pub possibly_undefined_from_try: bool,
    pub possibly_undefined: bool,
    pub ignore_nullable_issues: bool,
    pub ignore_falsable_issues: bool,
    pub from_template_default: bool,
    pub populated: bool,
}

impl Hash for TUnion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for t in self.types.as_ref() {
            t.hash(state);
        }
    }
}

impl TUnion {
    /// The primary constructor for creating a TUnion from a Cow.
    ///
    /// This is the most basic way to create a TUnion and is used by both the
    /// zero-allocation static helpers and the `from_vec` constructor.
    pub fn new(types: Cow<'static, [TAtomic]>) -> TUnion {
        TUnion {
            types,
            had_template: false,
            by_reference: false,
            reference_free: false,
            possibly_undefined_from_try: false,
            possibly_undefined: false,
            ignore_nullable_issues: false,
            ignore_falsable_issues: false,
            from_template_default: false,
            populated: false,
        }
    }

    /// Creates a TUnion from an owned Vec, performing necessary cleanup.
    ///
    /// This preserves the original logic for cleaning up dynamically created unions,
    /// such as removing redundant `never` types.
    pub fn from_vec(mut types: Vec<TAtomic>) -> TUnion {
        if cfg!(debug_assertions) {
            if types.is_empty() {
                panic!("TUnion::new() should not be called with an empty Vec.");
            }

            if types.len() > 1
                && types.iter().any(|atomic| {
                    atomic.is_never()
                        || atomic.map_generic_parameter_constraint(|constraint| constraint.is_never()).unwrap_or(false)
                })
            {
                panic!("TUnion::new() was called with a mix of 'never' and other types: {types:#?}")
            }
        } else {
            // If we have more than one type, 'never' is redundant and can be removed,
            // as the union `A|never` is simply `A`.
            if types.len() > 1 {
                types.retain(|atomic| {
                    !atomic.is_never()
                        && !atomic.map_generic_parameter_constraint(|constraint| constraint.is_never()).unwrap_or(false)
                });
            }

            // If the vector was originally empty, or contained only 'never' types
            // which were removed, ensure the final union is `never`.
            if types.is_empty() {
                types.push(TAtomic::Never);
            }
        }

        Self::new(Cow::Owned(types))
    }

    /// Creates a TUnion from a single atomic type, which can be either
    /// borrowed from a static source or owned.
    ///
    /// This function is a key optimization point. When passed a `Cow::Borrowed`,
    /// it creates the TUnion without any heap allocation.
    pub fn from_single(atomic: Cow<'static, TAtomic>) -> TUnion {
        let types_cow = match atomic {
            Cow::Borrowed(borrowed_atomic) => Cow::Borrowed(std::slice::from_ref(borrowed_atomic)),
            Cow::Owned(owned_atomic) => Cow::Owned(vec![owned_atomic]),
        };

        TUnion::new(types_cow)
    }

    /// Creates a TUnion from a single owned atomic type.
    pub fn from_atomic(atomic: TAtomic) -> TUnion {
        TUnion::new(Cow::Owned(vec![atomic]))
    }

    pub fn set_possibly_undefined(&mut self, possibly_undefined: bool, from_try: Option<bool>) {
        let from_try = from_try.unwrap_or(self.possibly_undefined_from_try);

        self.possibly_undefined = possibly_undefined;
        self.possibly_undefined_from_try = from_try;
    }

    /// Creates a new TUnion with the same properties as the original, but with a new set of types.
    pub fn clone_with_types(&self, types: Vec<TAtomic>) -> TUnion {
        TUnion {
            types: Cow::Owned(types),
            had_template: self.had_template,
            by_reference: self.by_reference,
            reference_free: self.reference_free,
            possibly_undefined_from_try: self.possibly_undefined_from_try,
            possibly_undefined: self.possibly_undefined,
            ignore_falsable_issues: self.ignore_falsable_issues,
            ignore_nullable_issues: self.ignore_nullable_issues,
            from_template_default: self.from_template_default,
            populated: self.populated,
        }
    }

    pub fn to_non_nullable(&self) -> TUnion {
        TUnion {
            types: Cow::Owned(self.get_non_nullable_types()),
            had_template: self.had_template,
            by_reference: self.by_reference,
            reference_free: self.reference_free,
            possibly_undefined_from_try: self.possibly_undefined_from_try,
            possibly_undefined: self.possibly_undefined,
            ignore_falsable_issues: self.ignore_falsable_issues,
            ignore_nullable_issues: self.ignore_nullable_issues,
            from_template_default: self.from_template_default,
            populated: self.populated,
        }
    }

    pub fn to_truthy(&self) -> TUnion {
        TUnion {
            types: Cow::Owned(self.get_truthy_types()),
            had_template: self.had_template,
            by_reference: self.by_reference,
            reference_free: self.reference_free,
            possibly_undefined_from_try: self.possibly_undefined_from_try,
            possibly_undefined: self.possibly_undefined,
            ignore_falsable_issues: self.ignore_falsable_issues,
            ignore_nullable_issues: self.ignore_nullable_issues,
            from_template_default: self.from_template_default,
            populated: self.populated,
        }
    }

    pub fn get_non_nullable_types(&self) -> Vec<TAtomic> {
        self.types
            .iter()
            .filter_map(|t| match t {
                TAtomic::Null | TAtomic::Void => None,
                TAtomic::GenericParameter(parameter) => Some(TAtomic::GenericParameter(TGenericParameter {
                    parameter_name: parameter.parameter_name,
                    defining_entity: parameter.defining_entity,
                    intersection_types: parameter.intersection_types.clone(),
                    constraint: Box::new(parameter.constraint.to_non_nullable()),
                })),
                TAtomic::Mixed(mixed) => Some(TAtomic::Mixed(mixed.with_is_non_null(true))),
                atomic => Some(atomic.clone()),
            })
            .collect()
    }

    pub fn get_truthy_types(&self) -> Vec<TAtomic> {
        self.types
            .iter()
            .filter_map(|t| match t {
                TAtomic::GenericParameter(parameter) => Some(TAtomic::GenericParameter(TGenericParameter {
                    parameter_name: parameter.parameter_name,
                    defining_entity: parameter.defining_entity,
                    intersection_types: parameter.intersection_types.clone(),
                    constraint: Box::new(parameter.constraint.to_truthy()),
                })),
                TAtomic::Mixed(mixed) => Some(TAtomic::Mixed(mixed.with_truthiness(TMixedTruthiness::Truthy))),
                atomic => {
                    if atomic.is_falsy() {
                        None
                    } else {
                        Some(atomic.clone())
                    }
                }
            })
            .collect()
    }

    /// Adds `null` to the union type, making it nullable.
    pub fn as_nullable(mut self) -> TUnion {
        let types = self.types.to_mut();

        types.iter_mut().for_each(|atomic| {
            if let TAtomic::Mixed(mixed) = atomic {
                *mixed = mixed.with_is_non_null(false);
            }
        });

        if !types.iter().any(|atomic| atomic.is_null() || atomic.is_mixed()) {
            types.push(TAtomic::Null);
        }

        self
    }

    /// Removes a specific atomic type from the union.
    pub fn remove_type(&mut self, bad_type: &TAtomic) {
        self.types.to_mut().retain(|t| t != bad_type);
    }

    /// Replaces a specific atomic type in the union with a new type.
    pub fn replace_type(&mut self, remove_type: &TAtomic, add_type: TAtomic) {
        let types = self.types.to_mut();

        if let Some(index) = types.iter().position(|t| t == remove_type) {
            types[index] = add_type;
        } else {
            types.push(add_type);
        }
    }

    pub fn is_int(&self) -> bool {
        for atomic in self.types.as_ref() {
            if !atomic.is_int() {
                return false;
            }
        }

        true
    }

    pub fn has_int_or_float(&self) -> bool {
        for atomic in self.types.as_ref() {
            if atomic.is_int_or_float() {
                return true;
            }
        }

        false
    }

    pub fn has_int_and_float(&self) -> bool {
        let mut has_int = false;
        let mut has_float = false;

        for atomic in self.types.as_ref() {
            if atomic.is_int() {
                has_int = true;
            } else if atomic.is_float() {
                has_float = true;
            } else if atomic.is_int_or_float() {
                has_int = true;
                has_float = true;
            }

            if has_int && has_float {
                return true;
            }
        }

        false
    }

    pub fn has_int_and_string(&self) -> bool {
        let mut has_int = false;
        let mut has_string = false;

        for atomic in self.types.as_ref() {
            if atomic.is_int() {
                has_int = true;
            } else if atomic.is_string() {
                has_string = true;
            } else if atomic.is_array_key() {
                has_int = true;
                has_string = true;
            }

            if has_int && has_string {
                return true;
            }
        }

        false
    }

    pub fn has_int(&self) -> bool {
        for atomic in self.types.as_ref() {
            if atomic.is_int() || atomic.is_array_key() || atomic.is_numeric() {
                return true;
            }
        }

        false
    }

    pub fn has_float(&self) -> bool {
        for atomic in self.types.as_ref() {
            if atomic.is_float() {
                return true;
            }
        }

        false
    }

    pub fn is_array_key(&self) -> bool {
        for atomic in self.types.as_ref() {
            if atomic.is_array_key() {
                continue;
            }

            return false;
        }

        true
    }

    pub fn is_any_string(&self) -> bool {
        for atomic in self.types.as_ref() {
            if !atomic.is_any_string() {
                return false;
            }
        }

        true
    }

    pub fn is_string(&self) -> bool {
        self.types.iter().all(|t| t.is_string()) && !self.types.is_empty()
    }

    pub fn is_always_array_key(&self, ignore_never: bool) -> bool {
        self.types.iter().all(|atomic| match atomic {
            TAtomic::Never => ignore_never,
            TAtomic::Scalar(scalar) => matches!(
                scalar,
                TScalar::ArrayKey | TScalar::Integer(_) | TScalar::String(_) | TScalar::ClassLikeString(_)
            ),
            TAtomic::GenericParameter(generic_parameter) => {
                generic_parameter.constraint.is_always_array_key(ignore_never)
            }
            _ => false,
        })
    }

    pub fn is_non_empty_string(&self) -> bool {
        self.types.iter().all(|t| t.is_non_empty_string()) && !self.types.is_empty()
    }

    pub fn is_empty_array(&self) -> bool {
        self.types.iter().all(|t| t.is_empty_array()) && !self.types.is_empty()
    }

    pub fn has_string(&self) -> bool {
        self.types.iter().any(|t| t.is_string()) && !self.types.is_empty()
    }

    pub fn is_float(&self) -> bool {
        self.types.iter().all(|t| t.is_float()) && !self.types.is_empty()
    }

    pub fn is_bool(&self) -> bool {
        self.types.iter().all(|t| t.is_bool()) && !self.types.is_empty()
    }

    pub fn is_never(&self) -> bool {
        self.types.iter().all(|t| t.is_never()) || self.types.is_empty()
    }

    pub fn is_never_template(&self) -> bool {
        self.types.iter().all(|t| t.is_templated_as_never()) && !self.types.is_empty()
    }

    pub fn is_placeholder(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Placeholder)) && !self.types.is_empty()
    }

    pub fn is_true(&self) -> bool {
        self.types.iter().all(|t| t.is_true()) && !self.types.is_empty()
    }

    pub fn is_false(&self) -> bool {
        self.types.iter().all(|t| t.is_false()) && !self.types.is_empty()
    }

    pub fn is_nonnull(&self) -> bool {
        self.types.len() == 1 && matches!(self.types[0], TAtomic::Mixed(mixed) if mixed.is_non_null())
    }

    pub fn is_numeric(&self) -> bool {
        self.types.iter().all(|t| t.is_numeric()) && !self.types.is_empty()
    }

    pub fn is_int_or_float(&self) -> bool {
        self.types.iter().all(|t| t.is_int_or_float()) && !self.types.is_empty()
    }

    pub fn is_mixed(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Mixed(_))) && !self.types.is_empty()
    }

    pub fn is_mixed_template(&self) -> bool {
        self.types.iter().all(|t| t.is_templated_as_mixed()) && !self.types.is_empty()
    }

    pub fn has_mixed(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Mixed(_))) && !self.types.is_empty()
    }

    pub fn has_mixed_template(&self) -> bool {
        self.types.iter().any(|t| t.is_templated_as_mixed()) && !self.types.is_empty()
    }

    pub fn has_nullable_mixed(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Mixed(mixed) if !mixed.is_non_null())) && !self.types.is_empty()
    }

    pub fn has_void(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Void)) && !self.types.is_empty()
    }

    pub fn has_null(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Null)) && !self.types.is_empty()
    }

    pub fn has_nullish(&self) -> bool {
        self.types.iter().any(|t| match t {
            TAtomic::Null | TAtomic::Void => true,
            TAtomic::Mixed(mixed) => !mixed.is_non_null(),
            TAtomic::GenericParameter(parameter) => parameter.constraint.has_nullish(),
            _ => false,
        }) && !self.types.is_empty()
    }

    pub fn is_nullable_mixed(&self) -> bool {
        if self.types.len() != 1 {
            return false;
        }

        match &self.types[0] {
            TAtomic::Mixed(mixed) => !mixed.is_non_null(),
            _ => false,
        }
    }

    pub fn is_falsy_mixed(&self) -> bool {
        if self.types.len() != 1 {
            return false;
        }

        matches!(&self.types[0], &TAtomic::Mixed(mixed) if mixed.is_falsy())
    }

    pub fn is_vanilla_mixed(&self) -> bool {
        if self.types.len() != 1 {
            return false;
        }

        matches!(&self.types[0], TAtomic::Mixed(mixed) if mixed.is_vanilla())
    }

    pub fn has_template_or_static(&self) -> bool {
        for atomic in self.types.as_ref() {
            if let TAtomic::GenericParameter(_) = atomic {
                return true;
            }

            if let TAtomic::Object(TObject::Named(named_object)) = atomic {
                if named_object.is_this() {
                    return true;
                }

                if let Some(intersections) = named_object.get_intersection_types() {
                    for intersection in intersections {
                        if let TAtomic::GenericParameter(_) = intersection {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    pub fn has_template(&self) -> bool {
        for atomic in self.types.as_ref() {
            if let TAtomic::GenericParameter(_) = atomic {
                return true;
            }

            if let Some(intersections) = atomic.get_intersection_types() {
                for intersection in intersections {
                    if let TAtomic::GenericParameter(_) = intersection {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn has_template_types(&self) -> bool {
        let all_child_nodes = self.get_all_child_nodes();

        for child_node in all_child_nodes {
            if let TypeRef::Atomic(
                TAtomic::GenericParameter(_)
                | TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic { .. })),
            ) = child_node
            {
                return true;
            }
        }

        false
    }

    pub fn get_template_types(&self) -> Vec<&TAtomic> {
        let all_child_nodes = self.get_all_child_nodes();

        let mut template_types = Vec::new();

        for child_node in all_child_nodes {
            if let TypeRef::Atomic(inner) = child_node {
                match inner {
                    TAtomic::GenericParameter(_) => {
                        template_types.push(inner);
                    }
                    TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic { .. })) => {
                        template_types.push(inner);
                    }
                    _ => {}
                }
            }
        }

        template_types
    }

    pub fn is_objecty(&self) -> bool {
        for atomic in self.types.as_ref() {
            if let &TAtomic::Object(_) = atomic {
                continue;
            }

            if let TAtomic::Callable(callable) = atomic
                && callable.get_signature().is_none_or(|signature| signature.is_closure())
            {
                continue;
            }

            return false;
        }

        true
    }

    pub fn is_generator(&self) -> bool {
        for atomic in self.types.as_ref() {
            if atomic.is_generator() {
                continue;
            }

            return false;
        }

        true
    }

    pub fn extends_or_implements(&self, codebase: &CodebaseMetadata, interface: &str) -> bool {
        for atomic in self.types.as_ref() {
            if !atomic.extends_or_implements(codebase, interface) {
                return false;
            }
        }

        true
    }

    pub fn is_generic_parameter(&self) -> bool {
        self.types.len() == 1 && matches!(self.types[0], TAtomic::GenericParameter(_))
    }

    pub fn get_generic_parameter_constraint(&self) -> Option<&TUnion> {
        if self.is_generic_parameter()
            && let TAtomic::GenericParameter(parameter) = &self.types[0]
        {
            return Some(&parameter.constraint);
        }

        None
    }

    pub fn is_null(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Null)) && !self.types.is_empty()
    }

    pub fn is_nullable(&self) -> bool {
        self.types.iter().any(|t| match t {
            TAtomic::Null => self.types.len() >= 2,
            _ => false,
        })
    }

    pub fn is_void(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Void)) && !self.types.is_empty()
    }

    pub fn is_voidable(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Void)) && !self.types.is_empty()
    }

    pub fn has_resource(&self) -> bool {
        self.types.iter().any(|t| t.is_resource())
    }

    pub fn is_resource(&self) -> bool {
        self.types.iter().all(|t| t.is_resource()) && !self.types.is_empty()
    }

    pub fn is_array(&self) -> bool {
        self.types.iter().all(|t| t.is_array()) && !self.types.is_empty()
    }

    pub fn is_list(&self) -> bool {
        self.types.iter().all(|t| t.is_list()) && !self.types.is_empty()
    }

    pub fn is_keyed_array(&self) -> bool {
        self.types.iter().all(|t| t.is_keyed_array()) && !self.types.is_empty()
    }

    pub fn is_falsable(&self) -> bool {
        self.types.len() >= 2 && self.types.iter().any(|t| t.is_false())
    }

    pub fn has_bool(&self) -> bool {
        self.types.iter().any(|t| t.is_bool() || t.is_generic_scalar()) && !self.types.is_empty()
    }

    /// Checks if the union explicitly contains the generic `scalar` type.
    ///
    /// This is a specific check for the `scalar` type itself, not for a
    /// combination of types that would form a scalar (e.g., `int|string|bool|float`).
    /// For that, see `has_scalar_combination`.
    pub fn has_scalar(&self) -> bool {
        self.types.iter().any(|atomic| atomic.is_generic_scalar())
    }

    /// Checks if the union contains a combination of types that is equivalent
    /// to the generic `scalar` type (i.e., contains `int`, `float`, `bool`, and `string`).
    pub fn has_scalar_combination(&self) -> bool {
        const HAS_INT: u8 = 1 << 0;
        const HAS_FLOAT: u8 = 1 << 1;
        const HAS_BOOL: u8 = 1 << 2;
        const HAS_STRING: u8 = 1 << 3;
        const ALL_SCALARS: u8 = HAS_INT | HAS_FLOAT | HAS_BOOL | HAS_STRING;

        let mut flags = 0u8;

        for atomic in self.types.as_ref() {
            if atomic.is_int() {
                flags |= HAS_INT;
            } else if atomic.is_float() {
                flags |= HAS_FLOAT;
            } else if atomic.is_bool() {
                flags |= HAS_BOOL;
            } else if atomic.is_string() {
                flags |= HAS_STRING;
            } else if atomic.is_array_key() {
                flags |= HAS_INT | HAS_STRING;
            } else if atomic.is_numeric() {
                // We don't add `string` as `numeric-string` does not contain `string` type
                flags |= HAS_INT | HAS_FLOAT;
            } else if atomic.is_generic_scalar() {
                return true;
            }

            // Early exit if we've already found all scalar types
            if flags == ALL_SCALARS {
                return true;
            }
        }

        flags == ALL_SCALARS
    }
    pub fn has_array_key(&self) -> bool {
        self.types.iter().any(|atomic| atomic.is_array_key())
    }

    pub fn has_iterable(&self) -> bool {
        self.types.iter().any(|atomic| atomic.is_iterable()) && !self.types.is_empty()
    }

    pub fn has_array(&self) -> bool {
        self.types.iter().any(|atomic| atomic.is_array()) && !self.types.is_empty()
    }

    pub fn has_traversable(&self, codebase: &CodebaseMetadata) -> bool {
        self.types.iter().any(|atomic| atomic.is_traversable(codebase)) && !self.types.is_empty()
    }

    pub fn has_array_key_like(&self) -> bool {
        self.types.iter().any(|atomic| atomic.is_array_key() || atomic.is_int() || atomic.is_string())
    }

    pub fn has_numeric(&self) -> bool {
        self.types.iter().any(|atomic| atomic.is_numeric()) && !self.types.is_empty()
    }

    pub fn is_always_truthy(&self) -> bool {
        self.types.iter().all(|atomic| atomic.is_truthy()) && !self.types.is_empty()
    }

    pub fn is_always_falsy(&self) -> bool {
        self.types.iter().all(|atomic| atomic.is_falsy()) && !self.types.is_empty()
    }

    pub fn is_literal_of(&self, other: &TUnion) -> bool {
        let Some(other_atomic_type) = other.types.first() else {
            return false;
        };

        match other_atomic_type {
            TAtomic::Scalar(TScalar::String(_)) => {
                for self_atomic_type in self.types.as_ref() {
                    if self_atomic_type.is_string_of_literal_origin() {
                        continue;
                    }

                    return false;
                }

                true
            }
            TAtomic::Scalar(TScalar::Integer(_)) => {
                for self_atomic_type in self.types.as_ref() {
                    if self_atomic_type.is_literal_int() {
                        continue;
                    }

                    return false;
                }

                true
            }
            TAtomic::Scalar(TScalar::Float(_)) => {
                for self_atomic_type in self.types.as_ref() {
                    if self_atomic_type.is_literal_float() {
                        continue;
                    }

                    return false;
                }

                true
            }
            _ => false,
        }
    }

    pub fn all_literals(&self) -> bool {
        self.types
            .iter()
            .all(|atomic| atomic.is_string_of_literal_origin() || atomic.is_literal_int() || atomic.is_literal_float())
    }

    pub fn has_static_object(&self) -> bool {
        self.types
            .iter()
            .any(|atomic| matches!(atomic, TAtomic::Object(TObject::Named(named_object)) if named_object.is_this()))
    }

    pub fn is_static_object(&self) -> bool {
        self.types
            .iter()
            .all(|atomic| matches!(atomic, TAtomic::Object(TObject::Named(named_object)) if named_object.is_this()))
    }

    #[inline]
    pub fn is_single(&self) -> bool {
        self.types.len() == 1
    }

    #[inline]
    pub fn get_single_string(&self) -> Option<&TString> {
        if self.is_single()
            && let TAtomic::Scalar(TScalar::String(string)) = &self.types[0]
        {
            Some(string)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_single_array(&self) -> Option<&TArray> {
        if self.is_single()
            && let TAtomic::Array(array) = &self.types[0]
        {
            Some(array)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_single_bool(&self) -> Option<&TBool> {
        if self.is_single()
            && let TAtomic::Scalar(TScalar::Bool(bool)) = &self.types[0]
        {
            Some(bool)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_single_named_object(&self) -> Option<&TNamedObject> {
        if self.is_single()
            && let TAtomic::Object(TObject::Named(named_object)) = &self.types[0]
        {
            Some(named_object)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_single(&self) -> &TAtomic {
        &self.types[0]
    }

    #[inline]
    pub fn get_single_owned(self) -> TAtomic {
        self.types[0].to_owned()
    }

    #[inline]
    pub fn is_named_object(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Object(TObject::Named(_))))
    }

    pub fn is_enum(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Object(TObject::Enum(_))))
    }

    pub fn is_enum_case(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Object(TObject::Enum(r#enum)) if r#enum.case.is_some()))
    }

    pub fn is_single_enum_case(&self) -> bool {
        self.is_single()
            && self.types.iter().all(|t| matches!(t, TAtomic::Object(TObject::Enum(r#enum)) if r#enum.case.is_some()))
    }

    #[inline]
    pub fn has_named_object(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Object(TObject::Named(_))))
    }

    #[inline]
    pub fn has_object(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Object(TObject::Any)))
    }

    #[inline]
    pub fn has_callable(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Callable(_)))
    }

    #[inline]
    pub fn is_callable(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Callable(_)))
    }

    #[inline]
    pub fn has_object_type(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Object(_)))
    }

    /// Return a vector of pairs containing the enum name, and their case name
    /// if specified.
    pub fn get_enum_cases(&self) -> Vec<(Atom, Option<Atom>)> {
        self.types
            .iter()
            .filter_map(|t| match t {
                TAtomic::Object(TObject::Enum(enum_object)) => Some((enum_object.name, enum_object.case)),
                _ => None,
            })
            .collect()
    }

    pub fn get_single_int(&self) -> Option<TInteger> {
        if self.is_single() { self.get_single().get_integer() } else { None }
    }

    pub fn get_single_literal_int_value(&self) -> Option<i64> {
        if self.is_single() { self.get_single().get_literal_int_value() } else { None }
    }

    pub fn get_single_maximum_int_value(&self) -> Option<i64> {
        if self.is_single() { self.get_single().get_maximum_int_value() } else { None }
    }

    pub fn get_single_minimum_int_value(&self) -> Option<i64> {
        if self.is_single() { self.get_single().get_minimum_int_value() } else { None }
    }

    pub fn get_single_literal_float_value(&self) -> Option<f64> {
        if self.is_single() { self.get_single().get_literal_float_value() } else { None }
    }

    pub fn get_single_literal_string_value(&self) -> Option<&str> {
        if self.is_single() { self.get_single().get_literal_string_value() } else { None }
    }

    pub fn get_single_class_string_value(&self) -> Option<Atom> {
        if self.is_single() { self.get_single().get_class_string_value() } else { None }
    }

    pub fn get_single_array_key(&self) -> Option<ArrayKey> {
        if self.is_single() { self.get_single().to_array_key() } else { None }
    }

    pub fn get_single_key_of_array_like(&self) -> Option<TUnion> {
        if !self.is_single() {
            return None;
        }

        match self.get_single() {
            TAtomic::Array(array) => match array {
                TArray::List(_) => Some(get_int()),
                TArray::Keyed(keyed_array) => match &keyed_array.parameters {
                    Some((k, _)) => Some(*k.clone()),
                    None => Some(get_arraykey()),
                },
            },
            _ => None,
        }
    }

    pub fn get_single_value_of_array_like(&self) -> Option<Cow<'_, TUnion>> {
        if !self.is_single() {
            return None;
        }

        match self.get_single() {
            TAtomic::Array(array) => match array {
                TArray::List(list) => Some(Cow::Borrowed(&list.element_type)),
                TArray::Keyed(keyed_array) => match &keyed_array.parameters {
                    Some((_, v)) => Some(Cow::Borrowed(v)),
                    None => Some(Cow::Owned(get_mixed())),
                },
            },
            _ => None,
        }
    }

    pub fn get_literal_ints(&self) -> Vec<&TAtomic> {
        self.types.iter().filter(|a| a.is_literal_int()).collect()
    }

    pub fn get_literal_strings(&self) -> Vec<&TAtomic> {
        self.types.iter().filter(|a| a.is_known_literal_string()).collect()
    }

    pub fn get_literal_string_values(&self) -> Vec<Option<Atom>> {
        self.get_literal_strings()
            .into_iter()
            .map(|atom| match atom {
                TAtomic::Scalar(TScalar::String(TString { literal: Some(TStringLiteral::Value(value)), .. })) => {
                    Some(*value)
                }
                _ => None,
            })
            .collect()
    }

    pub fn has_literal_float(&self) -> bool {
        self.types.iter().any(|atomic| match atomic {
            TAtomic::Scalar(scalar) => scalar.is_literal_float(),
            _ => false,
        })
    }

    pub fn has_literal_int(&self) -> bool {
        self.types.iter().any(|atomic| match atomic {
            TAtomic::Scalar(scalar) => scalar.is_literal_int(),
            _ => false,
        })
    }

    pub fn has_literal_string(&self) -> bool {
        self.types.iter().any(|atomic| match atomic {
            TAtomic::Scalar(scalar) => scalar.is_known_literal_string(),
            _ => false,
        })
    }

    pub fn has_literal_value(&self) -> bool {
        self.types.iter().any(|atomic| match atomic {
            TAtomic::Scalar(scalar) => scalar.is_literal_value(),
            _ => false,
        })
    }

    pub fn accepts_false(&self) -> bool {
        self.types.iter().any(|t| match t {
            TAtomic::GenericParameter(parameter) => parameter.constraint.accepts_false(),
            TAtomic::Mixed(mixed) if !mixed.is_truthy() => true,
            TAtomic::Scalar(TScalar::Generic | TScalar::Bool(TBool { value: None | Some(false) })) => true,
            _ => false,
        })
    }

    pub fn accepts_null(&self) -> bool {
        self.types.iter().any(|t| match t {
            TAtomic::GenericParameter(generic_parameter) => generic_parameter.constraint.accepts_null(),
            TAtomic::Mixed(mixed) if !mixed.is_non_null() => true,
            TAtomic::Null => true,
            _ => false,
        })
    }
}

impl TType for TUnion {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        self.types.iter().map(TypeRef::Atomic).collect()
    }

    fn needs_population(&self) -> bool {
        !self.populated && self.types.iter().any(|v| v.needs_population())
    }

    fn is_expandable(&self) -> bool {
        if self.types.is_empty() {
            return true;
        }

        self.types.iter().any(|t| t.is_expandable())
    }

    fn get_id(&self) -> Atom {
        let len = self.types.len();

        let mut atomic_ids: Vec<Atom> = self
            .types
            .as_ref()
            .iter()
            .map(|atomic| {
                let id = atomic.get_id();
                if atomic.has_intersection_types() && len > 1 { concat_atom!("(", id.as_str(), ")") } else { id }
            })
            .collect();

        if len <= 1 {
            return atomic_ids.pop().unwrap_or_else(empty_atom);
        }

        atomic_ids.sort_unstable();
        let mut result = atomic_ids[0];
        for id in &atomic_ids[1..] {
            result = concat_atom!(result.as_str(), "|", id.as_str());
        }

        result
    }
}

impl PartialEq for TUnion {
    fn eq(&self, other: &TUnion) -> bool {
        if self.reference_free != other.reference_free
            || self.by_reference != other.by_reference
            || self.had_template != other.had_template
            || self.possibly_undefined_from_try != other.possibly_undefined_from_try
            || self.possibly_undefined != other.possibly_undefined
            || self.ignore_falsable_issues != other.ignore_falsable_issues
            || self.ignore_nullable_issues != other.ignore_nullable_issues
            || self.from_template_default != other.from_template_default
        {
            return false;
        }

        let len = self.types.len();
        if len != other.types.len() {
            return false;
        }

        for i in 0..len {
            let mut has_match = false;
            for j in 0..len {
                if self.types[i] == other.types[j] {
                    has_match = true;
                    break;
                }
            }

            if !has_match {
                return false;
            }
        }

        true
    }
}

pub fn populate_union_type(
    unpopulated_union: &mut TUnion,
    codebase_symbols: &Symbols,
    reference_source: Option<&ReferenceSource>,
    symbol_references: &mut SymbolReferences,
    force: bool,
) {
    if unpopulated_union.populated && !force {
        return;
    }

    if !unpopulated_union.needs_population() {
        return;
    }

    unpopulated_union.populated = true;
    let unpopulated_atomics = unpopulated_union.types.to_mut();
    for unpopulated_atomic in unpopulated_atomics {
        match unpopulated_atomic {
            TAtomic::Scalar(TScalar::ClassLikeString(
                TClassLikeString::Generic { constraint, .. } | TClassLikeString::OfType { constraint, .. },
            )) => {
                let mut new_constraint = (**constraint).clone();

                populate_atomic_type(&mut new_constraint, codebase_symbols, reference_source, symbol_references, force);

                *constraint = Box::new(new_constraint);
            }
            _ => {
                populate_atomic_type(unpopulated_atomic, codebase_symbols, reference_source, symbol_references, force);
            }
        }
    }
}
