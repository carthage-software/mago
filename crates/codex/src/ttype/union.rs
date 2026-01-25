use std::borrow::Cow;
use std::hash::Hash;
use std::hash::Hasher;

use bitflags::bitflags;
use derivative::Derivative;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use mago_atom::empty_atom;

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
use crate::ttype::atomic::object::with_properties::TObjectWithProperties;
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

bitflags! {
    /// Flags representing various properties of a type union.
    ///
    /// This replaces 9 individual boolean fields with a compact 16-bit representation,
    /// reducing memory usage from 9 bytes to 2 bytes per TUnion instance.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct UnionFlags: u16 {
        /// Indicates the union had a template type at some point.
        const HAD_TEMPLATE = 1 << 0;
        /// Indicates the value is passed by reference.
        const BY_REFERENCE = 1 << 1;
        /// Indicates no references exist to this type.
        const REFERENCE_FREE = 1 << 2;
        /// Indicates the type may be undefined due to a try block.
        const POSSIBLY_UNDEFINED_FROM_TRY = 1 << 3;
        /// Indicates the type may be undefined.
        const POSSIBLY_UNDEFINED = 1 << 4;
        /// Indicates nullable issues should be ignored for this type.
        const IGNORE_NULLABLE_ISSUES = 1 << 5;
        /// Indicates falsable issues should be ignored for this type.
        const IGNORE_FALSABLE_ISSUES = 1 << 6;
        /// Indicates the type came from a template default value.
        const FROM_TEMPLATE_DEFAULT = 1 << 7;
        /// Indicates the type has been populated with codebase information.
        const POPULATED = 1 << 8;
        /// Indicates the null in this union came from nullsafe short-circuit.
        const NULLSAFE_NULL = 1 << 9;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Derivative, PartialOrd, Ord)]
pub struct TUnion {
    pub types: Cow<'static, [TAtomic]>,
    pub flags: UnionFlags,
}

impl Hash for TUnion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for t in self.types.as_ref() {
            t.hash(state);
        }
    }
}

impl TUnion {
    /// The primary constructor for creating a `TUnion` from a Cow.
    ///
    /// This is the most basic way to create a `TUnion` and is used by both the
    /// zero-allocation static helpers and the `from_vec` constructor.
    #[must_use]
    pub fn new(types: Cow<'static, [TAtomic]>) -> TUnion {
        TUnion { types, flags: UnionFlags::empty() }
    }

    /// Creates a `TUnion` from an owned Vec, performing necessary cleanup.
    ///
    /// This preserves the original logic for cleaning up dynamically created unions,
    /// such as removing redundant `never` types.
    ///
    /// # Panics
    ///
    /// In debug builds, panics if:
    /// - The input Vec is empty (unions must contain at least one type)
    /// - The input contains a mix of `never` types with other types (invalid union construction)
    #[must_use]
    pub fn from_vec(mut types: Vec<TAtomic>) -> TUnion {
        if cfg!(debug_assertions) {
            assert!(
                !types.is_empty(),
                "TUnion::from_vec() received an empty Vec. This indicates a logic error \
                 in type construction - unions must contain at least one type. \
                 Consider using TAtomic::Never for empty/impossible types."
            );

            if types.len() > 1
                && types.iter().any(|atomic| {
                    atomic.is_never() || atomic.map_generic_parameter_constraint(TUnion::is_never).unwrap_or(false)
                })
            {
                panic!(
                    "TUnion::from_vec() received a mix of 'never' and other types. \
                     This indicates a logic error - 'never' should be filtered out before \
                     creating unions since (A | never) = A. Types received: {types:#?}"
                )
            }
        } else {
            // If we have more than one type, 'never' is redundant and can be removed,
            // as the union `A|never` is simply `A`.
            if types.len() > 1 {
                types.retain(|atomic| {
                    !atomic.is_never() && !atomic.map_generic_parameter_constraint(TUnion::is_never).unwrap_or(false)
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

    /// Creates a `TUnion` from a single atomic type, which can be either
    /// borrowed from a static source or owned.
    ///
    /// This function is a key optimization point. When passed a `Cow::Borrowed`,
    /// it creates the `TUnion` without any heap allocation.
    #[must_use]
    pub fn from_single(atomic: Cow<'static, TAtomic>) -> TUnion {
        let types_cow = match atomic {
            Cow::Borrowed(borrowed_atomic) => Cow::Borrowed(std::slice::from_ref(borrowed_atomic)),
            Cow::Owned(owned_atomic) => Cow::Owned(vec![owned_atomic]),
        };

        TUnion::new(types_cow)
    }

    /// Creates a `TUnion` from a single owned atomic type.
    #[must_use]
    pub fn from_atomic(atomic: TAtomic) -> TUnion {
        TUnion::new(Cow::Owned(vec![atomic]))
    }

    #[inline]
    pub fn set_possibly_undefined(&mut self, possibly_undefined: bool, from_try: Option<bool>) {
        let from_try = from_try.unwrap_or(self.flags.contains(UnionFlags::POSSIBLY_UNDEFINED_FROM_TRY));

        self.flags.set(UnionFlags::POSSIBLY_UNDEFINED, possibly_undefined);
        self.flags.set(UnionFlags::POSSIBLY_UNDEFINED_FROM_TRY, from_try);
    }

    #[inline]
    #[must_use]
    pub const fn had_template(&self) -> bool {
        self.flags.contains(UnionFlags::HAD_TEMPLATE)
    }

    #[inline]
    #[must_use]
    pub const fn by_reference(&self) -> bool {
        self.flags.contains(UnionFlags::BY_REFERENCE)
    }

    #[inline]
    #[must_use]
    pub const fn reference_free(&self) -> bool {
        self.flags.contains(UnionFlags::REFERENCE_FREE)
    }

    #[inline]
    #[must_use]
    pub const fn possibly_undefined_from_try(&self) -> bool {
        self.flags.contains(UnionFlags::POSSIBLY_UNDEFINED_FROM_TRY)
    }

    #[inline]
    #[must_use]
    pub const fn possibly_undefined(&self) -> bool {
        self.flags.contains(UnionFlags::POSSIBLY_UNDEFINED)
    }

    #[inline]
    #[must_use]
    pub const fn ignore_nullable_issues(&self) -> bool {
        self.flags.contains(UnionFlags::IGNORE_NULLABLE_ISSUES)
    }

    #[inline]
    #[must_use]
    pub const fn ignore_falsable_issues(&self) -> bool {
        self.flags.contains(UnionFlags::IGNORE_FALSABLE_ISSUES)
    }

    #[inline]
    #[must_use]
    pub const fn from_template_default(&self) -> bool {
        self.flags.contains(UnionFlags::FROM_TEMPLATE_DEFAULT)
    }

    #[inline]
    #[must_use]
    pub const fn populated(&self) -> bool {
        self.flags.contains(UnionFlags::POPULATED)
    }

    #[inline]
    #[must_use]
    pub const fn has_nullsafe_null(&self) -> bool {
        self.flags.contains(UnionFlags::NULLSAFE_NULL)
    }

    #[inline]
    pub fn set_had_template(&mut self, value: bool) {
        self.flags.set(UnionFlags::HAD_TEMPLATE, value);
    }

    #[inline]
    pub fn set_by_reference(&mut self, value: bool) {
        self.flags.set(UnionFlags::BY_REFERENCE, value);
    }

    #[inline]
    pub fn set_reference_free(&mut self, value: bool) {
        self.flags.set(UnionFlags::REFERENCE_FREE, value);
    }

    #[inline]
    pub fn set_possibly_undefined_from_try(&mut self, value: bool) {
        self.flags.set(UnionFlags::POSSIBLY_UNDEFINED_FROM_TRY, value);
    }

    #[inline]
    pub fn set_ignore_nullable_issues(&mut self, value: bool) {
        self.flags.set(UnionFlags::IGNORE_NULLABLE_ISSUES, value);
    }

    #[inline]
    pub fn set_ignore_falsable_issues(&mut self, value: bool) {
        self.flags.set(UnionFlags::IGNORE_FALSABLE_ISSUES, value);
    }

    #[inline]
    pub fn set_from_template_default(&mut self, value: bool) {
        self.flags.set(UnionFlags::FROM_TEMPLATE_DEFAULT, value);
    }

    #[inline]
    pub fn set_populated(&mut self, value: bool) {
        self.flags.set(UnionFlags::POPULATED, value);
    }

    #[inline]
    pub fn set_nullsafe_null(&mut self, value: bool) {
        self.flags.set(UnionFlags::NULLSAFE_NULL, value);
    }

    /// Creates a new `TUnion` with the same properties as the original, but with a new set of types.
    #[must_use]
    pub fn clone_with_types(&self, types: Vec<TAtomic>) -> TUnion {
        TUnion { types: Cow::Owned(types), flags: self.flags }
    }

    #[must_use]
    pub fn to_non_nullable(&self) -> TUnion {
        TUnion { types: Cow::Owned(self.get_non_nullable_types()), flags: self.flags }
    }

    #[must_use]
    pub fn to_truthy(&self) -> TUnion {
        TUnion { types: Cow::Owned(self.get_truthy_types()), flags: self.flags }
    }

    #[must_use]
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

    #[must_use]
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
    #[must_use]
    pub fn as_nullable(mut self) -> TUnion {
        let types = self.types.to_mut();

        for atomic in types.iter_mut() {
            if let TAtomic::Mixed(mixed) = atomic {
                *mixed = mixed.with_is_non_null(false);
            }
        }

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

    #[must_use]
    pub fn is_int(&self) -> bool {
        for atomic in self.types.as_ref() {
            if !atomic.is_int() {
                return false;
            }
        }

        true
    }

    #[must_use]
    pub fn has_int_or_float(&self) -> bool {
        for atomic in self.types.as_ref() {
            if atomic.is_int_or_float() {
                return true;
            }
        }

        false
    }

    #[must_use]
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

    #[must_use]
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

    #[must_use]
    pub fn has_int(&self) -> bool {
        for atomic in self.types.as_ref() {
            if atomic.is_int() || atomic.is_array_key() || atomic.is_numeric() {
                return true;
            }
        }

        false
    }

    #[must_use]
    pub fn has_float(&self) -> bool {
        for atomic in self.types.as_ref() {
            if atomic.is_float() {
                return true;
            }
        }

        false
    }

    #[must_use]
    pub fn is_array_key(&self) -> bool {
        for atomic in self.types.as_ref() {
            if atomic.is_array_key() {
                continue;
            }

            return false;
        }

        true
    }

    #[must_use]
    pub fn is_any_string(&self) -> bool {
        for atomic in self.types.as_ref() {
            if !atomic.is_any_string() {
                return false;
            }
        }

        true
    }

    pub fn is_string(&self) -> bool {
        self.types.iter().all(TAtomic::is_string) && !self.types.is_empty()
    }

    #[must_use]
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
        self.types.iter().all(TAtomic::is_non_empty_string) && !self.types.is_empty()
    }

    pub fn is_empty_array(&self) -> bool {
        self.types.iter().all(TAtomic::is_empty_array) && !self.types.is_empty()
    }

    pub fn has_string(&self) -> bool {
        self.types.iter().any(TAtomic::is_string) && !self.types.is_empty()
    }

    pub fn is_float(&self) -> bool {
        self.types.iter().all(TAtomic::is_float) && !self.types.is_empty()
    }

    pub fn is_bool(&self) -> bool {
        self.types.iter().all(TAtomic::is_bool) && !self.types.is_empty()
    }

    pub fn is_never(&self) -> bool {
        self.types.iter().all(TAtomic::is_never) || self.types.is_empty()
    }

    pub fn is_never_template(&self) -> bool {
        self.types.iter().all(TAtomic::is_templated_as_never) && !self.types.is_empty()
    }

    #[must_use]
    pub fn is_placeholder(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Placeholder)) && !self.types.is_empty()
    }

    pub fn is_true(&self) -> bool {
        self.types.iter().all(TAtomic::is_true) && !self.types.is_empty()
    }

    pub fn is_false(&self) -> bool {
        self.types.iter().all(TAtomic::is_false) && !self.types.is_empty()
    }

    #[must_use]
    pub fn is_nonnull(&self) -> bool {
        self.types.len() == 1 && matches!(self.types[0], TAtomic::Mixed(mixed) if mixed.is_non_null())
    }

    pub fn is_numeric(&self) -> bool {
        self.types.iter().all(TAtomic::is_numeric) && !self.types.is_empty()
    }

    pub fn is_int_or_float(&self) -> bool {
        self.types.iter().all(TAtomic::is_int_or_float) && !self.types.is_empty()
    }

    /// Returns `Some(true)` if all types are effectively int, `Some(false)` if all are effectively float,
    /// or `None` if mixed or neither. Handles unions like `1|2` (all int) or `3.4|4.5` (all float).
    #[must_use]
    pub fn effective_int_or_float(&self) -> Option<bool> {
        let mut result: Option<bool> = None;
        for atomic in self.types.as_ref() {
            match atomic.effective_int_or_float() {
                Some(is_int) => {
                    if let Some(prev) = result {
                        if prev != is_int {
                            return None;
                        }
                    } else {
                        result = Some(is_int);
                    }
                }
                None => return None,
            }
        }

        result
    }

    #[must_use]
    pub fn is_mixed(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Mixed(_))) && !self.types.is_empty()
    }

    pub fn is_mixed_template(&self) -> bool {
        self.types.iter().all(TAtomic::is_templated_as_mixed) && !self.types.is_empty()
    }

    #[must_use]
    pub fn has_mixed(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Mixed(_))) && !self.types.is_empty()
    }

    pub fn has_mixed_template(&self) -> bool {
        self.types.iter().any(TAtomic::is_templated_as_mixed) && !self.types.is_empty()
    }

    #[must_use]
    pub fn has_nullable_mixed(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Mixed(mixed) if !mixed.is_non_null())) && !self.types.is_empty()
    }

    #[must_use]
    pub fn has_void(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Void)) && !self.types.is_empty()
    }

    #[must_use]
    pub fn has_null(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Null)) && !self.types.is_empty()
    }

    #[must_use]
    pub fn has_nullish(&self) -> bool {
        self.types.iter().any(|t| match t {
            TAtomic::Null | TAtomic::Void => true,
            TAtomic::Mixed(mixed) => !mixed.is_non_null(),
            TAtomic::GenericParameter(parameter) => parameter.constraint.has_nullish(),
            _ => false,
        }) && !self.types.is_empty()
    }

    #[must_use]
    pub fn is_nullable_mixed(&self) -> bool {
        if self.types.len() != 1 {
            return false;
        }

        match &self.types[0] {
            TAtomic::Mixed(mixed) => !mixed.is_non_null(),
            _ => false,
        }
    }

    #[must_use]
    pub fn is_falsy_mixed(&self) -> bool {
        if self.types.len() != 1 {
            return false;
        }

        matches!(&self.types[0], &TAtomic::Mixed(mixed) if mixed.is_falsy())
    }

    #[must_use]
    pub fn is_vanilla_mixed(&self) -> bool {
        if self.types.len() != 1 {
            return false;
        }

        self.types[0].is_vanilla_mixed()
    }

    #[must_use]
    pub fn is_templated_as_vanilla_mixed(&self) -> bool {
        if self.types.len() != 1 {
            return false;
        }

        self.types[0].is_templated_as_vanilla_mixed()
    }

    #[must_use]
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

    #[must_use]
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

    #[must_use]
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

    #[must_use]
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
                && callable.get_signature().is_none_or(super::atomic::callable::TCallableSignature::is_closure)
            {
                continue;
            }

            return false;
        }

        true
    }

    #[must_use]
    pub fn is_generator(&self) -> bool {
        for atomic in self.types.as_ref() {
            if atomic.is_generator() {
                continue;
            }

            return false;
        }

        true
    }

    #[must_use]
    pub fn extends_or_implements(&self, codebase: &CodebaseMetadata, interface: &str) -> bool {
        for atomic in self.types.as_ref() {
            if !atomic.extends_or_implements(codebase, interface) {
                return false;
            }
        }

        true
    }

    #[must_use]
    pub fn is_generic_parameter(&self) -> bool {
        self.types.len() == 1 && matches!(self.types[0], TAtomic::GenericParameter(_))
    }

    #[must_use]
    pub fn get_generic_parameter_constraint(&self) -> Option<&TUnion> {
        if self.is_generic_parameter()
            && let TAtomic::GenericParameter(parameter) = &self.types[0]
        {
            return Some(&parameter.constraint);
        }

        None
    }

    #[must_use]
    pub fn is_null(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Null)) && !self.types.is_empty()
    }

    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.types.iter().any(|t| match t {
            TAtomic::Null => self.types.len() >= 2,
            TAtomic::GenericParameter(param) => param.constraint.is_nullable(),
            _ => false,
        })
    }

    #[must_use]
    pub fn is_void(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Void)) && !self.types.is_empty()
    }

    #[must_use]
    pub fn is_voidable(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Void)) && !self.types.is_empty()
    }

    pub fn has_resource(&self) -> bool {
        self.types.iter().any(TAtomic::is_resource)
    }

    pub fn is_resource(&self) -> bool {
        self.types.iter().all(TAtomic::is_resource) && !self.types.is_empty()
    }

    pub fn is_array(&self) -> bool {
        self.types.iter().all(TAtomic::is_array) && !self.types.is_empty()
    }

    pub fn is_list(&self) -> bool {
        self.types.iter().all(TAtomic::is_list) && !self.types.is_empty()
    }

    pub fn is_vanilla_array(&self) -> bool {
        self.types.iter().all(TAtomic::is_vanilla_array) && !self.types.is_empty()
    }

    pub fn is_keyed_array(&self) -> bool {
        self.types.iter().all(TAtomic::is_keyed_array) && !self.types.is_empty()
    }

    pub fn is_falsable(&self) -> bool {
        self.types.len() >= 2 && self.types.iter().any(TAtomic::is_false)
    }

    #[must_use]
    pub fn has_bool(&self) -> bool {
        self.types.iter().any(|t| t.is_bool() || t.is_generic_scalar()) && !self.types.is_empty()
    }

    /// Checks if the union explicitly contains the generic `scalar` type.
    ///
    /// This is a specific check for the `scalar` type itself, not for a
    /// combination of types that would form a scalar (e.g., `int|string|bool|float`).
    /// For that, see `has_scalar_combination`.
    pub fn has_scalar(&self) -> bool {
        self.types.iter().any(TAtomic::is_generic_scalar)
    }

    /// Checks if the union contains a combination of types that is equivalent
    /// to the generic `scalar` type (i.e., contains `int`, `float`, `bool`, and `string`).
    #[must_use]
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
        self.types.iter().any(TAtomic::is_array_key)
    }

    pub fn has_iterable(&self) -> bool {
        self.types.iter().any(TAtomic::is_iterable) && !self.types.is_empty()
    }

    pub fn has_array(&self) -> bool {
        self.types.iter().any(TAtomic::is_array) && !self.types.is_empty()
    }

    #[must_use]
    pub fn has_traversable(&self, codebase: &CodebaseMetadata) -> bool {
        self.types.iter().any(|atomic| atomic.is_traversable(codebase)) && !self.types.is_empty()
    }

    #[must_use]
    pub fn has_array_key_like(&self) -> bool {
        self.types.iter().any(|atomic| atomic.is_array_key() || atomic.is_int() || atomic.is_string())
    }

    pub fn has_numeric(&self) -> bool {
        self.types.iter().any(TAtomic::is_numeric) && !self.types.is_empty()
    }

    pub fn is_always_truthy(&self) -> bool {
        self.types.iter().all(TAtomic::is_truthy) && !self.types.is_empty()
    }

    pub fn is_always_falsy(&self) -> bool {
        self.types.iter().all(TAtomic::is_falsy) && !self.types.is_empty()
    }

    #[must_use]
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

    #[must_use]
    pub fn all_literals(&self) -> bool {
        self.types
            .iter()
            .all(|atomic| atomic.is_string_of_literal_origin() || atomic.is_literal_int() || atomic.is_literal_float())
    }

    #[must_use]
    pub fn has_static_object(&self) -> bool {
        self.types
            .iter()
            .any(|atomic| matches!(atomic, TAtomic::Object(TObject::Named(named_object)) if named_object.is_this()))
    }

    #[must_use]
    pub fn is_static_object(&self) -> bool {
        self.types
            .iter()
            .all(|atomic| matches!(atomic, TAtomic::Object(TObject::Named(named_object)) if named_object.is_this()))
    }

    #[inline]
    #[must_use]
    pub fn is_single(&self) -> bool {
        self.types.len() == 1
    }

    #[inline]
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn get_single_shaped_object(&self) -> Option<&TObjectWithProperties> {
        if self.is_single()
            && let TAtomic::Object(TObject::WithProperties(shaped_object)) = &self.types[0]
        {
            Some(shaped_object)
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn get_single(&self) -> &TAtomic {
        &self.types[0]
    }

    #[inline]
    #[must_use]
    pub fn get_single_owned(self) -> TAtomic {
        self.types[0].clone()
    }

    #[inline]
    #[must_use]
    pub fn is_named_object(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Object(TObject::Named(_))))
    }

    #[must_use]
    pub fn is_enum(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Object(TObject::Enum(_))))
    }

    #[must_use]
    pub fn is_enum_case(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Object(TObject::Enum(r#enum)) if r#enum.case.is_some()))
    }

    #[must_use]
    pub fn is_single_enum_case(&self) -> bool {
        self.is_single()
            && self.types.iter().all(|t| matches!(t, TAtomic::Object(TObject::Enum(r#enum)) if r#enum.case.is_some()))
    }

    #[inline]
    #[must_use]
    pub fn has_named_object(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Object(TObject::Named(_))))
    }

    #[inline]
    #[must_use]
    pub fn has_object(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Object(TObject::Any | TObject::WithProperties(_))))
    }

    #[inline]
    #[must_use]
    pub fn has_callable(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Callable(_)))
    }

    #[inline]
    #[must_use]
    pub fn is_callable(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Callable(_)))
    }

    #[inline]
    #[must_use]
    pub fn has_object_type(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Object(_)))
    }

    /// Return a vector of pairs containing the enum name, and their case name
    /// if specified.
    #[must_use]
    pub fn get_enum_cases(&self) -> Vec<(Atom, Option<Atom>)> {
        self.types
            .iter()
            .filter_map(|t| match t {
                TAtomic::Object(TObject::Enum(enum_object)) => Some((enum_object.name, enum_object.case)),
                _ => None,
            })
            .collect()
    }

    #[must_use]
    pub fn get_single_int(&self) -> Option<TInteger> {
        if self.is_single() { self.get_single().get_integer() } else { None }
    }

    #[must_use]
    pub fn get_single_literal_int_value(&self) -> Option<i64> {
        if self.is_single() { self.get_single().get_literal_int_value() } else { None }
    }

    #[must_use]
    pub fn get_single_maximum_int_value(&self) -> Option<i64> {
        if self.is_single() { self.get_single().get_maximum_int_value() } else { None }
    }

    #[must_use]
    pub fn get_single_minimum_int_value(&self) -> Option<i64> {
        if self.is_single() { self.get_single().get_minimum_int_value() } else { None }
    }

    #[must_use]
    pub fn get_single_literal_float_value(&self) -> Option<f64> {
        if self.is_single() { self.get_single().get_literal_float_value() } else { None }
    }

    #[must_use]
    pub fn get_single_literal_string_value(&self) -> Option<&str> {
        if self.is_single() { self.get_single().get_literal_string_value() } else { None }
    }

    #[must_use]
    pub fn get_single_class_string_value(&self) -> Option<Atom> {
        if self.is_single() { self.get_single().get_class_string_value() } else { None }
    }

    #[must_use]
    pub fn get_single_array_key(&self) -> Option<ArrayKey> {
        if self.is_single() { self.get_single().to_array_key() } else { None }
    }

    #[must_use]
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

    #[must_use]
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

    #[must_use]
    pub fn get_literal_ints(&self) -> Vec<&TAtomic> {
        self.types.iter().filter(|a| a.is_literal_int()).collect()
    }

    #[must_use]
    pub fn get_literal_strings(&self) -> Vec<&TAtomic> {
        self.types.iter().filter(|a| a.is_known_literal_string()).collect()
    }

    #[must_use]
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

    #[must_use]
    pub fn has_literal_float(&self) -> bool {
        self.types.iter().any(|atomic| match atomic {
            TAtomic::Scalar(scalar) => scalar.is_literal_float(),
            _ => false,
        })
    }

    #[must_use]
    pub fn has_literal_int(&self) -> bool {
        self.types.iter().any(|atomic| match atomic {
            TAtomic::Scalar(scalar) => scalar.is_literal_int(),
            _ => false,
        })
    }

    #[must_use]
    pub fn has_literal_string(&self) -> bool {
        self.types.iter().any(|atomic| match atomic {
            TAtomic::Scalar(scalar) => scalar.is_known_literal_string(),
            _ => false,
        })
    }

    #[must_use]
    pub fn has_literal_value(&self) -> bool {
        self.types.iter().any(|atomic| match atomic {
            TAtomic::Scalar(scalar) => scalar.is_literal_value(),
            _ => false,
        })
    }

    #[must_use]
    pub fn accepts_false(&self) -> bool {
        self.types.iter().any(|t| match t {
            TAtomic::GenericParameter(parameter) => parameter.constraint.accepts_false(),
            TAtomic::Mixed(mixed) if !mixed.is_truthy() => true,
            TAtomic::Scalar(TScalar::Generic | TScalar::Bool(TBool { value: None | Some(false) })) => true,
            _ => false,
        })
    }

    #[must_use]
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
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        self.types.iter().map(TypeRef::Atomic).collect()
    }

    fn needs_population(&self) -> bool {
        !self.flags.contains(UnionFlags::POPULATED) && self.types.iter().any(super::TType::needs_population)
    }

    fn is_expandable(&self) -> bool {
        if self.types.is_empty() {
            return true;
        }

        self.types.iter().any(super::TType::is_expandable)
    }

    fn is_complex(&self) -> bool {
        self.types.len() > 3 || self.types.iter().any(super::TType::is_complex)
    }

    fn get_id(&self) -> Atom {
        let len = self.types.len();

        let mut atomic_ids: Vec<Atom> = self
            .types
            .as_ref()
            .iter()
            .map(|atomic| {
                let id = atomic.get_id();
                if atomic.is_generic_parameter() || atomic.has_intersection_types() && len > 1 {
                    concat_atom!("(", id.as_str(), ")")
                } else {
                    id
                }
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

    fn get_pretty_id_with_indent(&self, indent: usize) -> Atom {
        let len = self.types.len();

        if len <= 1 {
            return self.types.first().map_or_else(empty_atom, |atomic| atomic.get_pretty_id_with_indent(indent));
        }

        // Use multiline format for unions with more than 3 types
        if len > 3 {
            let mut atomic_ids: Vec<Atom> = self
                .types
                .as_ref()
                .iter()
                .map(|atomic| {
                    let id = atomic.get_pretty_id_with_indent(indent + 2);
                    if atomic.has_intersection_types() { concat_atom!("(", id.as_str(), ")") } else { id }
                })
                .collect();

            atomic_ids.sort_unstable();

            let mut result = String::new();
            result += &atomic_ids[0];
            for id in &atomic_ids[1..] {
                result += "\n";
                result += &" ".repeat(indent);
                result += "| ";
                result += id.as_str();
            }

            atom(&result)
        } else {
            // Use inline format for smaller unions
            let mut atomic_ids: Vec<Atom> = self
                .types
                .as_ref()
                .iter()
                .map(|atomic| {
                    let id = atomic.get_pretty_id_with_indent(indent);
                    if atomic.has_intersection_types() && len > 1 { concat_atom!("(", id.as_str(), ")") } else { id }
                })
                .collect();

            atomic_ids.sort_unstable();
            let mut result = atomic_ids[0];
            for id in &atomic_ids[1..] {
                result = concat_atom!(result.as_str(), " | ", id.as_str());
            }

            result
        }
    }
}

impl PartialEq for TUnion {
    fn eq(&self, other: &TUnion) -> bool {
        const SEMANTIC_FLAGS: UnionFlags = UnionFlags::HAD_TEMPLATE
            .union(UnionFlags::BY_REFERENCE)
            .union(UnionFlags::REFERENCE_FREE)
            .union(UnionFlags::POSSIBLY_UNDEFINED_FROM_TRY)
            .union(UnionFlags::POSSIBLY_UNDEFINED)
            .union(UnionFlags::IGNORE_NULLABLE_ISSUES)
            .union(UnionFlags::IGNORE_FALSABLE_ISSUES)
            .union(UnionFlags::FROM_TEMPLATE_DEFAULT);

        if self.flags.intersection(SEMANTIC_FLAGS) != other.flags.intersection(SEMANTIC_FLAGS) {
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
    if unpopulated_union.flags.contains(UnionFlags::POPULATED) && !force {
        return;
    }

    if !unpopulated_union.needs_population() {
        return;
    }

    unpopulated_union.flags.insert(UnionFlags::POPULATED);
    let unpopulated_atomics = unpopulated_union.types.to_mut();
    for unpopulated_atomic in unpopulated_atomics {
        match unpopulated_atomic {
            TAtomic::Scalar(TScalar::ClassLikeString(
                TClassLikeString::Generic { constraint, .. } | TClassLikeString::OfType { constraint, .. },
            )) => {
                let mut new_constraint = (**constraint).clone();

                populate_atomic_type(&mut new_constraint, codebase_symbols, reference_source, symbol_references, force);

                **constraint = new_constraint;
            }
            _ => {
                populate_atomic_type(unpopulated_atomic, codebase_symbols, reference_source, symbol_references, force);
            }
        }
    }
}
