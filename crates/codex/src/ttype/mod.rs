use std::borrow::Cow;

use mago_atom::Atom;
use mago_atom::atom;

use crate::get_class_like;
use crate::is_instance_of;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::misc::GenericParent;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::iterable::TIterable;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeStringKind;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::shared::*;
use crate::ttype::template::TemplateResult;
use crate::ttype::template::inferred_type_replacer;
use crate::ttype::union::TUnion;

pub mod atomic;
pub mod builder;
pub mod cast;
pub mod combination;
pub mod combiner;
pub mod comparator;
pub mod error;
pub mod expander;
pub mod resolution;
pub mod shared;
pub mod template;
pub mod union;

/// A reference to a type in the type system, which can be either a union or an atomic type.
#[derive(Clone, Copy, Debug)]
pub enum TypeRef<'a> {
    Union(&'a TUnion),
    Atomic(&'a TAtomic),
}

/// A trait to be implemented by all types in the type system.
pub trait TType {
    /// Returns a vector of child type nodes that this type contains.
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        vec![]
    }

    /// Returns a vector of all child type nodes, including nested ones.
    fn get_all_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        let mut child_nodes = self.get_child_nodes();
        let mut all_child_nodes = vec![];

        while let Some(child_node) = child_nodes.pop() {
            let new_child_nodes = match child_node {
                TypeRef::Union(union) => union.get_child_nodes(),
                TypeRef::Atomic(atomic) => atomic.get_child_nodes(),
            };

            all_child_nodes.push(child_node);

            child_nodes.extend(new_child_nodes);
        }

        all_child_nodes
    }

    /// Checks if this type can have intersection types (`&B&S`).
    fn can_be_intersected(&self) -> bool {
        false
    }

    /// Returns a slice of the additional intersection types (`&B&S`), if any. Contains boxed atomic types.
    fn get_intersection_types(&self) -> Option<&[TAtomic]> {
        None
    }

    /// Returns a mutable slice of the additional intersection types (`&B&S`), if any. Contains boxed atomic types.
    fn get_intersection_types_mut(&mut self) -> Option<&mut Vec<TAtomic>> {
        None
    }

    /// Checks if this type has intersection types.
    fn has_intersection_types(&self) -> bool {
        false
    }

    /// Adds an intersection type to this type.
    ///
    /// Returns `true` if the intersection type was added successfully,
    ///  or `false` if this type does not support intersection types.
    fn add_intersection_type(&mut self, _intersection_type: TAtomic) -> bool {
        false
    }

    fn needs_population(&self) -> bool;

    fn is_expandable(&self) -> bool;

    /// Return a human-readable atom for this type, which is
    /// suitable for use in error messages or debugging.
    ///
    /// The resulting identifier must be unique for the type,
    /// but it does not have to be globally unique.
    fn get_id(&self) -> Atom;
}

/// Implements the `TType` trait for `TypeRef`.
impl<'a> TType for TypeRef<'a> {
    fn get_child_nodes(&self) -> Vec<TypeRef<'a>> {
        match self {
            TypeRef::Union(ttype) => ttype.get_child_nodes(),
            TypeRef::Atomic(ttype) => ttype.get_child_nodes(),
        }
    }

    fn can_be_intersected(&self) -> bool {
        match self {
            TypeRef::Union(ttype) => ttype.can_be_intersected(),
            TypeRef::Atomic(ttype) => ttype.can_be_intersected(),
        }
    }

    fn get_intersection_types(&self) -> Option<&[TAtomic]> {
        match self {
            TypeRef::Union(ttype) => ttype.get_intersection_types(),
            TypeRef::Atomic(ttype) => ttype.get_intersection_types(),
        }
    }

    fn has_intersection_types(&self) -> bool {
        match self {
            TypeRef::Union(ttype) => ttype.has_intersection_types(),
            TypeRef::Atomic(ttype) => ttype.has_intersection_types(),
        }
    }

    fn needs_population(&self) -> bool {
        match self {
            TypeRef::Union(ttype) => ttype.needs_population(),
            TypeRef::Atomic(ttype) => ttype.needs_population(),
        }
    }

    fn is_expandable(&self) -> bool {
        match self {
            TypeRef::Union(ttype) => ttype.is_expandable(),
            TypeRef::Atomic(ttype) => ttype.is_expandable(),
        }
    }

    fn get_id(&self) -> Atom {
        match self {
            TypeRef::Union(ttype) => ttype.get_id(),
            TypeRef::Atomic(ttype) => ttype.get_id(),
        }
    }
}

impl<'a> From<&'a TUnion> for TypeRef<'a> {
    fn from(reference: &'a TUnion) -> Self {
        TypeRef::Union(reference)
    }
}

impl<'a> From<&'a TAtomic> for TypeRef<'a> {
    fn from(reference: &'a TAtomic) -> Self {
        TypeRef::Atomic(reference)
    }
}

/// Creates a `TUnion` from a `TInteger`, using a canonical static type where possible.
///
/// This function is a key optimization point. It checks if the provided `TInteger`
/// matches a common, reusable form (like "any integer" or "a positive integer").
/// If it does, it returns a zero-allocation `TUnion` that borrows a static,
/// shared instance.
///
/// For specific literal values or ranges that do not have a canonical static
/// representation, it falls back to creating a new, owned `TUnion`, which
/// involves a heap allocation.
pub fn get_union_from_integer(integer: &TInteger) -> TUnion {
    if integer.is_unspecified() {
        return get_int();
    }

    if integer.is_positive() {
        return get_positive_int();
    }

    if integer.is_negative() {
        return get_negative_int();
    }

    if integer.is_non_negative() {
        return get_non_negative_int();
    }

    if integer.is_non_positive() {
        return get_non_positive_int();
    }

    TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::Integer(*integer))))
}

#[inline]
pub fn wrap_atomic(tinner: TAtomic) -> TUnion {
    TUnion::from_single(Cow::Owned(tinner))
}

#[inline]
pub fn get_int() -> TUnion {
    TUnion::from_single(Cow::Borrowed(INT_ATOMIC))
}

#[inline]
pub fn get_positive_int() -> TUnion {
    TUnion::from_single(Cow::Borrowed(POSITIVE_INT_ATOMIC))
}

#[inline]
pub fn get_negative_int() -> TUnion {
    TUnion::from_single(Cow::Borrowed(NEGATIVE_INT_ATOMIC))
}

#[inline]
pub fn get_non_positive_int() -> TUnion {
    TUnion::from_single(Cow::Borrowed(NON_POSITIVE_INT_ATOMIC))
}

#[inline]
pub fn get_non_negative_int() -> TUnion {
    TUnion::from_single(Cow::Borrowed(NON_NEGATIVE_INT_ATOMIC))
}

#[inline]
pub fn get_int_range(from: Option<i64>, to: Option<i64>) -> TUnion {
    let atomic = match (from, to) {
        (Some(from), Some(to)) => TAtomic::Scalar(TScalar::Integer(TInteger::Range(from, to))),
        (Some(from), None) => {
            if 0 == from {
                return get_non_negative_int();
            }

            if 1 == from {
                return get_positive_int();
            }

            TAtomic::Scalar(TScalar::Integer(TInteger::From(from)))
        }
        (None, Some(to)) => {
            if 0 == to {
                return get_non_positive_int();
            }

            if -1 == to {
                return get_negative_int();
            }

            TAtomic::Scalar(TScalar::Integer(TInteger::To(to)))
        }
        (None, None) => return get_int(),
    };

    TUnion::from_single(Cow::Owned(atomic))
}

/// Returns a zero-allocation `TUnion` for the type `-1|0|1`.
#[inline]
pub fn get_signum_result() -> TUnion {
    TUnion::new(Cow::Borrowed(SIGNUM_RESULT_SLICE))
}

/// Returns a zero-allocation `TUnion` for the integer literal `1`.
#[inline]
pub fn get_one_int() -> TUnion {
    TUnion::from_single(Cow::Borrowed(ONE_INT_ATOMIC))
}

/// Returns a zero-allocation `TUnion` for the integer literal `0`.
#[inline]
pub fn get_zero_int() -> TUnion {
    TUnion::from_single(Cow::Borrowed(ZERO_INT_ATOMIC))
}

/// Returns a zero-allocation `TUnion` for the integer literal `-1`.
#[inline]
pub fn get_minus_one_int() -> TUnion {
    TUnion::from_single(Cow::Borrowed(MINUS_ONE_INT_ATOMIC))
}

#[inline]
pub fn get_literal_int(value: i64) -> TUnion {
    if value == 0 {
        return get_zero_int();
    }

    if value == 1 {
        return get_one_int();
    }

    if value == -1 {
        return get_minus_one_int();
    }

    TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::literal_int(value))))
}

#[inline]
pub fn get_int_or_float() -> TUnion {
    TUnion::new(Cow::Borrowed(INT_FLOAT_ATOMIC_SLICE))
}

#[inline]
pub fn get_int_or_string() -> TUnion {
    TUnion::new(Cow::Borrowed(INT_STRING_ATOMIC_SLICE))
}

#[inline]
pub fn get_nullable_int() -> TUnion {
    TUnion::new(Cow::Borrowed(NULL_INT_ATOMIC_SLICE))
}

#[inline]
pub fn get_nullable_float() -> TUnion {
    TUnion::new(Cow::Borrowed(NULL_FLOAT_ATOMIC_SLICE))
}

#[inline]
pub fn get_nullable_object() -> TUnion {
    TUnion::new(Cow::Borrowed(NULL_OBJECT_ATOMIC_SLICE))
}

#[inline]
pub fn get_nullable_string() -> TUnion {
    TUnion::new(Cow::Borrowed(NULL_STRING_ATOMIC_SLICE))
}

#[inline]
pub fn get_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(STRING_ATOMIC))
}

/// Returns a zero-allocation `TUnion` for a `string` with the specified properties.
///
/// This function maps all possible boolean property combinations to a canonical,
/// static `TAtomic` instance, avoiding heap allocations for common string types.
pub fn get_string_with_props(is_numeric: bool, is_truthy: bool, is_non_empty: bool, is_lowercase: bool) -> TUnion {
    let atomic_ref = match (is_numeric, is_truthy, is_non_empty, is_lowercase) {
        // is_numeric = true
        (true, true, _, _) => NUMERIC_TRUTHY_STRING_ATOMIC,
        (true, false, _, _) => NUMERIC_STRING_ATOMIC,
        // is_numeric = false, is_truthy = true
        (false, true, _, false) => TRUTHY_STRING_ATOMIC,
        (false, true, _, true) => TRUTHY_LOWERCASE_STRING_ATOMIC,
        // is_numeric = false, is_truthy = false
        (false, false, false, false) => STRING_ATOMIC,
        (false, false, false, true) => LOWERCASE_STRING_ATOMIC,
        (false, false, true, false) => NON_EMPTY_STRING_ATOMIC,
        (false, false, true, true) => NON_EMPTY_LOWERCASE_STRING_ATOMIC,
    };

    TUnion::from_single(Cow::Borrowed(atomic_ref))
}

#[inline]
pub fn get_literal_class_string(value: Atom) -> TUnion {
    TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::literal(value)))))
}

#[inline]
pub fn get_class_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(CLASS_STRING_ATOMIC))
}

#[inline]
pub fn get_class_string_of_type(constraint: TAtomic) -> TUnion {
    TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::class_string_of_type(
        constraint,
    )))))
}

#[inline]
pub fn get_interface_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(INTERFACE_STRING_ATOMIC))
}

#[inline]
pub fn get_interface_string_of_type(constraint: TAtomic) -> TUnion {
    TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::ClassLikeString(
        TClassLikeString::interface_string_of_type(constraint),
    ))))
}

#[inline]
pub fn get_enum_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(ENUM_STRING_ATOMIC))
}

#[inline]
pub fn get_enum_string_of_type(constraint: TAtomic) -> TUnion {
    TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::enum_string_of_type(
        constraint,
    )))))
}

#[inline]
pub fn get_trait_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(TRAIT_STRING_ATOMIC))
}

#[inline]
pub fn get_trait_string_of_type(constraint: TAtomic) -> TUnion {
    TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::trait_string_of_type(
        constraint,
    )))))
}

#[inline]
pub fn get_literal_string(value: Atom) -> TUnion {
    TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::literal_string(value))))
}

#[inline]
pub fn get_float() -> TUnion {
    TUnion::from_single(Cow::Borrowed(FLOAT_ATOMIC))
}

#[inline]
pub fn get_literal_float(v: f64) -> TUnion {
    TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::literal_float(v))))
}

#[inline]
pub fn get_mixed() -> TUnion {
    TUnion::from_single(Cow::Borrowed(MIXED_ATOMIC))
}

#[inline]
pub fn get_isset_from_mixed_mixed() -> TUnion {
    TUnion::from_single(Cow::Borrowed(ISSET_FROM_LOOP_MIXED_ATOMIC))
}

pub fn get_mixed_maybe_from_loop(from_loop_isset: bool) -> TUnion {
    if from_loop_isset { get_isset_from_mixed_mixed() } else { get_mixed() }
}

#[inline]
pub fn get_never() -> TUnion {
    TUnion::from_single(Cow::Borrowed(NEVER_ATOMIC))
}

#[inline]
pub fn get_resource() -> TUnion {
    TUnion::from_single(Cow::Borrowed(RESOURCE_ATOMIC))
}

#[inline]
pub fn get_closed_resource() -> TUnion {
    TUnion::from_single(Cow::Borrowed(CLOSED_RESOURCE_ATOMIC))
}

#[inline]
pub fn get_open_resource() -> TUnion {
    TUnion::from_single(Cow::Borrowed(OPEN_RESOURCE_ATOMIC))
}

#[inline]
pub fn get_placeholder() -> TUnion {
    TUnion::from_single(Cow::Borrowed(PLACEHOLDER_ATOMIC))
}

#[inline]
pub fn get_void() -> TUnion {
    TUnion::from_single(Cow::Borrowed(VOID_ATOMIC))
}

#[inline]
pub fn get_null() -> TUnion {
    TUnion::from_single(Cow::Borrowed(NULL_ATOMIC))
}

#[inline]
pub fn get_arraykey() -> TUnion {
    TUnion::from_single(Cow::Borrowed(ARRAYKEY_ATOMIC))
}

#[inline]
pub fn get_bool() -> TUnion {
    TUnion::from_single(Cow::Borrowed(BOOL_ATOMIC))
}

#[inline]
pub fn get_false() -> TUnion {
    TUnion::from_single(Cow::Borrowed(FALSE_ATOMIC))
}

#[inline]
pub fn get_true() -> TUnion {
    TUnion::from_single(Cow::Borrowed(TRUE_ATOMIC))
}

#[inline]
pub fn get_object() -> TUnion {
    TUnion::from_single(Cow::Borrowed(OBJECT_ATOMIC))
}

#[inline]
pub fn get_numeric() -> TUnion {
    TUnion::from_single(Cow::Borrowed(NUMERIC_ATOMIC))
}

#[inline]
pub fn get_numeric_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(NUMERIC_STRING_ATOMIC))
}

#[inline]
pub fn get_lowercase_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(LOWERCASE_STRING_ATOMIC))
}

#[inline]
pub fn get_non_empty_lowercase_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(NON_EMPTY_LOWERCASE_STRING_ATOMIC))
}

#[inline]
pub fn get_non_empty_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(NON_EMPTY_STRING_ATOMIC))
}

#[inline]
pub fn get_empty_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(&EMPTY_STRING_ATOMIC))
}

#[inline]
pub fn get_truthy_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(TRUTHY_STRING_ATOMIC))
}

#[inline]
pub fn get_unspecified_literal_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(UNSPECIFIED_LITERAL_STRING_ATOMIC))
}

#[inline]
pub fn get_non_empty_unspecified_literal_string() -> TUnion {
    TUnion::from_single(Cow::Borrowed(NON_EMPTY_UNSPECIFIED_LITERAL_STRING_ATOMIC))
}

#[inline]
pub fn get_scalar() -> TUnion {
    TUnion::from_single(Cow::Borrowed(SCALAR_ATOMIC))
}

#[inline]
pub fn get_nullable_scalar() -> TUnion {
    TUnion::new(Cow::Borrowed(NULL_SCALAR_ATOMIC_SLICE))
}

#[inline]
pub fn get_mixed_iterable() -> TUnion {
    TUnion::from_single(Cow::Borrowed(&MIXED_ITERABLE_ATOMIC))
}

#[inline]
pub fn get_empty_keyed_array() -> TUnion {
    TUnion::from_single(Cow::Borrowed(&EMPTY_KEYED_ARRAY_ATOMIC))
}

#[inline]
pub fn get_mixed_list() -> TUnion {
    get_list(get_mixed())
}

#[inline]
pub fn get_mixed_keyed_array() -> TUnion {
    get_keyed_array(get_arraykey(), get_mixed())
}

#[inline]
pub fn get_mixed_callable() -> TUnion {
    TUnion::from_single(Cow::Borrowed(&MIXED_CALLABLE_ATOMIC))
}

#[inline]
pub fn get_mixed_closure() -> TUnion {
    TUnion::from_single(Cow::Borrowed(&MIXED_CLOSURE_ATOMIC))
}

#[inline]
pub fn get_named_object(name: Atom, type_resolution_context: Option<&TypeResolutionContext>) -> TUnion {
    if let Some(type_resolution_context) = type_resolution_context
        && let Some(defining_entities) = type_resolution_context.get_template_definition(&name)
    {
        return wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
            kind: TClassLikeStringKind::Class,
            parameter_name: name,
            defining_entity: defining_entities[0].0,
            constraint: Box::new((*(defining_entities[0].1.get_single())).clone()),
        })));
    }

    wrap_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(name))))
}

#[inline]
pub fn get_iterable(key_parameter: TUnion, value_parameter: TUnion) -> TUnion {
    wrap_atomic(TAtomic::Iterable(TIterable::new(Box::new(key_parameter), Box::new(value_parameter))))
}

#[inline]
pub fn get_list(element_type: TUnion) -> TUnion {
    wrap_atomic(TAtomic::Array(TArray::List(TList::new(Box::new(element_type)))))
}

#[inline]
pub fn get_keyed_array(key_parameter: TUnion, value_parameter: TUnion) -> TUnion {
    wrap_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
        Box::new(key_parameter),
        Box::new(value_parameter),
    ))))
}

#[inline]
pub fn add_optional_union_type(base_type: TUnion, maybe_type: Option<&TUnion>, codebase: &CodebaseMetadata) -> TUnion {
    if let Some(type_2) = maybe_type { add_union_type(base_type, type_2, codebase, false) } else { base_type }
}

#[inline]
pub fn combine_optional_union_types(
    type_1: Option<&TUnion>,
    type_2: Option<&TUnion>,
    codebase: &CodebaseMetadata,
) -> TUnion {
    match (type_1, type_2) {
        (Some(type_1), Some(type_2)) => combine_union_types(type_1, type_2, codebase, false),
        (Some(type_1), None) => type_1.clone(),
        (None, Some(type_2)) => type_2.clone(),
        (None, None) => get_mixed(),
    }
}

#[inline]
pub fn combine_union_types(
    type_1: &TUnion,
    type_2: &TUnion,
    codebase: &CodebaseMetadata,
    overwrite_empty_array: bool,
) -> TUnion {
    if type_1 == type_2 {
        return type_1.clone();
    }

    let mut combined_type = if type_1.is_never() || type_1.is_never_template() {
        type_2.clone()
    } else if type_2.is_never() || type_2.is_never_template() {
        type_1.clone()
    } else if type_1.is_vanilla_mixed() && type_2.is_vanilla_mixed() {
        get_mixed()
    } else {
        let mut all_atomic_types = type_1.types.clone().into_owned();
        all_atomic_types.extend(type_2.types.clone().into_owned());

        let mut result = TUnion::from_vec(combiner::combine(all_atomic_types, codebase, overwrite_empty_array));

        if type_1.had_template && type_2.had_template {
            result.had_template = true;
        }

        if type_1.reference_free && type_2.reference_free {
            result.reference_free = true;
        }

        result
    };

    if type_1.possibly_undefined || type_2.possibly_undefined {
        combined_type.possibly_undefined = true;
    }

    if type_1.possibly_undefined_from_try || type_2.possibly_undefined_from_try {
        combined_type.possibly_undefined_from_try = true;
    }

    if type_1.ignore_falsable_issues || type_2.ignore_falsable_issues {
        combined_type.ignore_falsable_issues = true;
    }

    combined_type
}

#[inline]
pub fn add_union_type(
    mut base_type: TUnion,
    other_type: &TUnion,
    codebase: &CodebaseMetadata,
    overwrite_empty_array: bool,
) -> TUnion {
    if &base_type == other_type {
        base_type.possibly_undefined |= other_type.possibly_undefined;
        base_type.possibly_undefined_from_try |= other_type.possibly_undefined_from_try;
        base_type.ignore_falsable_issues |= other_type.ignore_falsable_issues;
        base_type.ignore_nullable_issues |= other_type.ignore_nullable_issues;

        return base_type;
    }

    base_type.types = if base_type.is_vanilla_mixed() && other_type.is_vanilla_mixed() {
        base_type.types
    } else {
        combine_union_types(&base_type, other_type, codebase, overwrite_empty_array).types
    };

    if !other_type.had_template {
        base_type.had_template = false;
    }

    if !other_type.reference_free {
        base_type.reference_free = false;
    }

    base_type.possibly_undefined |= other_type.possibly_undefined;
    base_type.possibly_undefined_from_try |= other_type.possibly_undefined_from_try;
    base_type.ignore_falsable_issues |= other_type.ignore_falsable_issues;
    base_type.ignore_nullable_issues |= other_type.ignore_nullable_issues;

    base_type
}

pub fn intersect_union_types(_type_1: &TUnion, _type_2: &TUnion, _codebase: &CodebaseMetadata) -> Option<TUnion> {
    None
}

pub fn get_iterable_parameters(atomic: &TAtomic, codebase: &CodebaseMetadata) -> Option<(TUnion, TUnion)> {
    if let Some(generator_parameters) = atomic.get_generator_parameters() {
        return Some((generator_parameters.0, generator_parameters.1));
    }

    let parameters = 'parameters: {
        match atomic {
            TAtomic::Iterable(iterable) => Some((iterable.get_key_type().clone(), iterable.get_value_type().clone())),
            TAtomic::Array(array_type) => Some(get_array_parameters(array_type, codebase)),
            TAtomic::Object(object) => {
                let name = object.get_name()?;
                let traversable = atom("traversable");

                let class_metadata = get_class_like(codebase, name)?;
                if !is_instance_of(codebase, &class_metadata.name, &traversable) {
                    break 'parameters None;
                }

                let traversable_metadata = get_class_like(codebase, &traversable)?;
                let key_template = traversable_metadata.template_types.first().map(|(name, _)| name)?;
                let value_template = traversable_metadata.template_types.get(1).map(|(name, _)| name)?;

                let key_type = get_specialized_template_type(
                    codebase,
                    key_template,
                    &traversable,
                    class_metadata,
                    object.get_type_parameters(),
                )
                .unwrap_or_else(get_mixed);

                let value_type = get_specialized_template_type(
                    codebase,
                    value_template,
                    &traversable,
                    class_metadata,
                    object.get_type_parameters(),
                )
                .unwrap_or_else(get_mixed);

                Some((key_type, value_type))
            }
            _ => None,
        }
    };

    if let Some((key_type, value_type)) = parameters {
        return Some((key_type, value_type));
    }

    if let Some(intersection_types) = atomic.get_intersection_types() {
        for intersection_type in intersection_types {
            if let Some((key_type, value_type)) = get_iterable_parameters(intersection_type, codebase) {
                return Some((key_type, value_type));
            }
        }
    }

    None
}

pub fn get_array_parameters(array_type: &TArray, codebase: &CodebaseMetadata) -> (TUnion, TUnion) {
    match array_type {
        TArray::Keyed(keyed_data) => {
            let mut key_types = vec![];
            let mut value_param;

            if let Some((key_param, value_p)) = &keyed_data.parameters {
                key_types.extend(key_param.types.clone().into_owned());
                value_param = (**value_p).clone();
            } else {
                key_types.push(TAtomic::Never);
                value_param = get_never();
            }

            if let Some(known_items) = &keyed_data.known_items {
                for (key, (_, item_type)) in known_items {
                    key_types.push(key.to_atomic());
                    value_param = add_union_type(value_param, item_type, codebase, false);
                }
            }

            let combined_key_types = combiner::combine(key_types, codebase, false);
            let key_param_union = TUnion::from_vec(combined_key_types);

            (key_param_union, value_param)
        }
        TArray::List(list_data) => {
            let mut key_types = vec![];
            let mut value_type = (*list_data.element_type).clone();

            if let Some(known_elements) = &list_data.known_elements {
                for (key_idx, (_, element_type)) in known_elements {
                    key_types.push(TAtomic::Scalar(TScalar::literal_int(*key_idx as i64)));

                    value_type = combine_union_types(element_type, &value_type, codebase, false);
                }
            }

            if key_types.is_empty() || !value_type.is_never() {
                if value_type.is_never() {
                    key_types.push(TAtomic::Never);
                } else {
                    key_types.push(TAtomic::Scalar(TScalar::Integer(TInteger::non_negative())));
                }
            }

            let key_type = TUnion::from_vec(combiner::combine(key_types, codebase, false));

            (key_type, value_type)
        }
    }
}

pub fn get_iterable_value_parameter(atomic: &TAtomic, codebase: &CodebaseMetadata) -> Option<TUnion> {
    if let Some(generator_parameters) = atomic.get_generator_parameters() {
        return Some(generator_parameters.1);
    }

    let parameter = match atomic {
        TAtomic::Iterable(iterable) => Some(iterable.get_value_type().clone()),
        TAtomic::Array(array_type) => Some(get_array_value_parameter(array_type, codebase)),
        TAtomic::Object(object) => {
            let name = object.get_name()?;
            let traversable = atom("traversable");

            let class_metadata = get_class_like(codebase, name)?;
            if !is_instance_of(codebase, &class_metadata.name, &traversable) {
                return None;
            }

            let traversable_metadata = get_class_like(codebase, &traversable)?;
            let value_template = traversable_metadata.template_types.get(1).map(|(name, _)| name)?;

            get_specialized_template_type(
                codebase,
                value_template,
                &traversable,
                class_metadata,
                object.get_type_parameters(),
            )
        }
        _ => None,
    };

    if let Some(value_param) = parameter {
        return Some(value_param);
    }

    if let Some(intersection_types) = atomic.get_intersection_types() {
        for intersection_type in intersection_types {
            if let Some(value_param) = get_iterable_value_parameter(intersection_type, codebase) {
                return Some(value_param);
            }
        }
    }

    None
}

pub fn get_array_value_parameter(array_type: &TArray, codebase: &CodebaseMetadata) -> TUnion {
    match array_type {
        TArray::Keyed(keyed_data) => {
            let mut value_param;

            if let Some((_, value_p)) = &keyed_data.parameters {
                value_param = (**value_p).clone();
            } else {
                value_param = get_never();
            }

            if let Some(known_items) = &keyed_data.known_items {
                for (_, item_type) in known_items.values() {
                    value_param = combine_union_types(item_type, &value_param, codebase, false);
                }
            }

            value_param
        }
        TArray::List(list_data) => {
            let mut value_param = (*list_data.element_type).clone();

            if let Some(known_elements) = &list_data.known_elements {
                for (_, element_type) in known_elements.values() {
                    value_param = combine_union_types(element_type, &value_param, codebase, false);
                }
            }

            value_param
        }
    }
}

/// Resolves a generic template from an ancestor class in the context of a descendant class.
///
/// This function correctly traverses the pre-calculated inheritance map to determine the
/// concrete type of a template parameter.
pub fn get_specialized_template_type(
    codebase: &CodebaseMetadata,
    template_name: &Atom,
    template_defining_class_id: &Atom,
    instantiated_class_metadata: &ClassLikeMetadata,
    instantiated_type_parameters: Option<&[TUnion]>,
) -> Option<TUnion> {
    let defining_class_metadata = get_class_like(codebase, template_defining_class_id)?;

    if defining_class_metadata.name == instantiated_class_metadata.name {
        let index = instantiated_class_metadata.get_template_index_for_name(template_name)?;

        let Some(instantiated_type_parameters) = instantiated_type_parameters else {
            let type_map = instantiated_class_metadata.get_template_type(template_name)?;

            return type_map.first().map(|(_, constraint)| constraint).cloned();
        };

        return instantiated_type_parameters.get(index).cloned();
    }

    let defining_template_type = defining_class_metadata.get_template_type(template_name)?;
    let template_union = TUnion::from_vec(
        defining_template_type
            .iter()
            .map(|(defining_entity, constraint)| {
                TAtomic::GenericParameter(TGenericParameter {
                    parameter_name: *template_name,
                    defining_entity: *defining_entity,
                    constraint: Box::new(constraint.clone()),
                    intersection_types: None,
                })
            })
            .collect::<Vec<_>>(),
    );

    let mut template_result = TemplateResult::default();
    for (defining_class, type_parameters_map) in &instantiated_class_metadata.template_extended_parameters {
        for (parameter_name, parameter_type) in type_parameters_map {
            template_result.add_lower_bound(
                *parameter_name,
                GenericParent::ClassLike(*defining_class),
                parameter_type.clone(),
            );
        }
    }

    let mut template_type = inferred_type_replacer::replace(&template_union, &template_result, codebase);
    if let Some(type_parameters) = instantiated_type_parameters {
        let mut template_result = TemplateResult::default();
        for (i, parameter_type) in type_parameters.iter().enumerate() {
            if let Some(parameter_name) = instantiated_class_metadata.get_template_name_for_index(i) {
                template_result.add_lower_bound(
                    parameter_name,
                    GenericParent::ClassLike(instantiated_class_metadata.name),
                    parameter_type.clone(),
                );
            }
        }

        if !template_result.lower_bounds.is_empty() {
            template_type = inferred_type_replacer::replace(&template_type, &template_result, codebase);
        }
    }

    Some(template_type)
}
