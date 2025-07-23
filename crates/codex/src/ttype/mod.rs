use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;

use crate::data_flow::node::DataFlowNode;
use crate::get_class_like;
use crate::is_instance_of;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::misc::GenericParent;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::callable::TCallableSignature;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::iterable::TIterable;
use crate::ttype::atomic::mixed::TMixed;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::resource::TResource;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeStringKind;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::resolution::TypeResolutionContext;
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

    /// Return a human-readable identifier for this type, which is
    /// suitable for use in error messages or debugging.
    ///
    /// The `interner` parameter is optional and can be used to resolve
    /// string identifiers to their actual names.
    ///
    /// The resulting identifier must be unique for the type,
    /// but it does not have to be globally unique.
    fn get_id(&self, interner: Option<&ThreadedInterner>) -> String;
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

    fn get_id(&self, interner: Option<&ThreadedInterner>) -> String {
        match self {
            TypeRef::Union(ttype) => ttype.get_id(interner),
            TypeRef::Atomic(ttype) => ttype.get_id(interner),
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

#[inline]
pub fn wrap_atomic(tinner: TAtomic) -> TUnion {
    TUnion::new(vec![tinner])
}

#[inline]
pub fn get_int() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::int()))
}

#[inline]
pub fn get_int_range(from: Option<i64>, to: Option<i64>) -> TUnion {
    match (from, to) {
        (Some(from), Some(to)) => wrap_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::Range(from, to)))),
        (Some(from), None) => wrap_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::From(from)))),
        (None, Some(to)) => wrap_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::To(to)))),
        (None, None) => get_int(),
    }
}

#[inline]
pub fn get_literal_int(value: i64) -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::literal_int(value)))
}

#[inline]
pub fn get_string() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::string()))
}

pub fn get_string_with_props(is_numeric: bool, is_truthy: bool, is_non_empty: bool) -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(is_numeric, is_truthy, is_non_empty))))
}

#[inline]
pub fn get_literal_class_string(value: StringIdentifier) -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::literal(value))))
}

#[inline]
pub fn get_class_string() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::class_string())))
}

#[inline]
pub fn get_class_string_of_type(constraint: TAtomic) -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::class_string_of_type(constraint))))
}

#[inline]
pub fn get_interface_string() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::interface_string())))
}

#[inline]
pub fn get_interface_string_of_type(constraint: TAtomic) -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::interface_string_of_type(constraint))))
}

#[inline]
pub fn get_enum_string() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::enum_string())))
}

#[inline]
pub fn get_enum_string_of_type(constraint: TAtomic) -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::enum_string_of_type(constraint))))
}

#[inline]
pub fn get_trait_string() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::trait_string())))
}

#[inline]
pub fn get_trait_string_of_type(constraint: TAtomic) -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::trait_string_of_type(constraint))))
}

#[inline]
pub fn get_literal_string(value: String) -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::literal_string(value)))
}

#[inline]
pub fn get_float() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::float()))
}

#[inline]
pub fn get_literal_float(v: f64) -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::literal_float(v)))
}

#[inline]
pub fn get_mixed() -> TUnion {
    wrap_atomic(TAtomic::Mixed(TMixed::vanilla()))
}

#[inline]
pub fn get_mixed_any() -> TUnion {
    wrap_atomic(TAtomic::Mixed(TMixed::any()))
}

pub fn get_mixed_maybe_from_loop(from_loop_isset: bool) -> TUnion {
    wrap_atomic(TAtomic::Mixed(TMixed::maybe_isset_from_loop(from_loop_isset)))
}

#[inline]
pub fn get_never() -> TUnion {
    wrap_atomic(TAtomic::Never)
}

#[inline]
pub fn get_resource() -> TUnion {
    wrap_atomic(TAtomic::Resource(TResource::new(None)))
}

#[inline]
pub fn get_closed_resource() -> TUnion {
    wrap_atomic(TAtomic::Resource(TResource::new(Some(true))))
}

#[inline]
pub fn get_open_resource() -> TUnion {
    wrap_atomic(TAtomic::Resource(TResource::new(Some(false))))
}

#[inline]
pub fn get_placeholder() -> TUnion {
    wrap_atomic(TAtomic::Placeholder)
}

#[inline]
pub fn get_void() -> TUnion {
    wrap_atomic(TAtomic::Void)
}

#[inline]
pub fn get_null() -> TUnion {
    wrap_atomic(TAtomic::Null)
}

#[inline]
pub fn get_num() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::Number))
}

#[inline]
pub fn get_arraykey() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::ArrayKey))
}

#[inline]
pub fn get_bool() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::bool()))
}

#[inline]
pub fn get_false() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::r#false()))
}

#[inline]
pub fn get_true() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::r#true()))
}

#[inline]
pub fn get_object() -> TUnion {
    wrap_atomic(TAtomic::Object(TObject::Any))
}

#[inline]
pub fn get_numeric() -> TUnion {
    TUnion::new(vec![TAtomic::Scalar(TScalar::Numeric)])
}

#[inline]
pub fn get_numeric_string() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(true, false, false))))
}

#[inline]
pub fn get_non_empty_string() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(false, false, true))))
}

#[inline]
pub fn get_truthy_string() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(false, true, false))))
}

#[inline]
pub fn get_unspecified_literal_string() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal())))
}

#[inline]
pub fn get_non_empty_unspecified_literal_string() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(false, false, true))))
}

#[inline]
pub fn get_named_object(
    interner: &ThreadedInterner,
    name: StringIdentifier,
    type_resolution_context: Option<&TypeResolutionContext>,
) -> TUnion {
    if let Some(type_resolution_context) = type_resolution_context {
        let name_str = interner.lookup(&name);
        if let Some(defining_entities) = type_resolution_context.get_template_definition(name_str) {
            return wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
                kind: TClassLikeStringKind::Class,
                parameter_name: name,
                defining_entity: defining_entities[0].0,
                constraint: Box::new((*(defining_entities[0].1.get_single())).clone()),
            })));
        }
    }

    wrap_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(name))))
}

#[inline]
pub fn get_scalar() -> TUnion {
    wrap_atomic(TAtomic::Scalar(TScalar::Generic))
}

#[inline]
pub fn get_mixed_iterable() -> TUnion {
    wrap_atomic(TAtomic::Iterable(TIterable::mixed()))
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
pub fn get_empty_keyed_array() -> TUnion {
    wrap_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new())))
}

#[inline]
pub fn get_keyed_array(key_parameter: TUnion, value_parameter: TUnion) -> TUnion {
    wrap_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
        Box::new(key_parameter),
        Box::new(value_parameter),
    ))))
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
    wrap_atomic(TAtomic::Callable(TCallable::Signature(TCallableSignature::mixed(false))))
}

#[inline]
pub fn get_mixed_closure() -> TUnion {
    wrap_atomic(TAtomic::Callable(TCallable::Signature(TCallableSignature::mixed(true))))
}

#[inline]
pub fn add_optional_union_type(
    base_type: TUnion,
    maybe_type: Option<&TUnion>,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
) -> TUnion {
    if let Some(type_2) = maybe_type { add_union_type(base_type, type_2, codebase, interner, false) } else { base_type }
}

#[inline]
pub fn combine_optional_union_types(
    type_1: Option<&TUnion>,
    type_2: Option<&TUnion>,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
) -> TUnion {
    if let Some(type_1) = type_1 {
        if let Some(type_2) = type_2 {
            combine_union_types(type_1, type_2, codebase, interner, false)
        } else {
            type_1.clone()
        }
    } else {
        type_2.unwrap().clone()
    }
}

#[inline]
pub fn extend_dataflow_uniquely(type_1_nodes: &mut Vec<DataFlowNode>, type_2_nodes: Vec<DataFlowNode>) {
    type_1_nodes.extend(type_2_nodes);
    type_1_nodes.sort_by(|a, b| a.id.cmp(&b.id));
    type_1_nodes.dedup_by(|a, b| a.id.eq(&b.id));
}

#[inline]
pub fn combine_union_types(
    type_1: &TUnion,
    type_2: &TUnion,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    overwrite_empty_array: bool,
) -> TUnion {
    if type_1 == type_2 {
        return type_1.clone();
    }

    let mut combined_type;

    if type_1.is_vanilla_mixed() && type_2.is_vanilla_mixed() {
        combined_type = get_mixed();
    } else {
        let mut all_atomic_types = type_1.types.clone();
        all_atomic_types.extend(type_2.types.clone());

        combined_type = TUnion::new(combiner::combine(all_atomic_types, codebase, interner, overwrite_empty_array));

        if type_1.had_template && type_2.had_template {
            combined_type.had_template = true;
        }

        if type_1.reference_free && type_2.reference_free {
            combined_type.reference_free = true;
        }
    }

    if type_1.possibly_undefined || type_2.possibly_undefined {
        combined_type.possibly_undefined = true;
    }

    if type_1.possibly_undefined_from_try || type_2.possibly_undefined_from_try {
        combined_type.possibly_undefined_from_try = true;
    }

    if type_1.ignore_falsable_issues || type_2.ignore_falsable_issues {
        combined_type.ignore_falsable_issues = true;
    }

    let type_1_parent_nodes_empty = type_1.parent_nodes.is_empty();
    let type_2_parent_nodes_empty = type_2.parent_nodes.is_empty();

    if !type_1_parent_nodes_empty || !type_2_parent_nodes_empty {
        if type_1_parent_nodes_empty {
            combined_type.parent_nodes.clone_from(&type_2.parent_nodes);
        } else if type_2_parent_nodes_empty {
            combined_type.parent_nodes.clone_from(&type_1.parent_nodes);
        } else {
            combined_type.parent_nodes.clone_from(&type_1.parent_nodes);
            extend_dataflow_uniquely(&mut combined_type.parent_nodes, type_2.parent_nodes.clone());
        }
    }

    combined_type
}

#[inline]
pub fn add_union_type(
    mut base_type: TUnion,
    other_type: &TUnion,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
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
        let mut all_atomic_types = base_type.types.clone();
        all_atomic_types.extend(other_type.types.clone());

        combiner::combine(all_atomic_types, codebase, interner, overwrite_empty_array)
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

    if !other_type.parent_nodes.is_empty() {
        extend_dataflow_uniquely(&mut base_type.parent_nodes, other_type.parent_nodes.clone());
    }

    base_type
}

pub fn intersect_union_types(_type_1: &TUnion, _type_2: &TUnion, _codebase: &CodebaseMetadata) -> Option<TUnion> {
    None
}

pub fn get_iterable_parameters(
    atomic: &TAtomic,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
) -> Option<(TUnion, TUnion)> {
    if let Some(generator_parameters) = atomic.get_generator_parameters(interner) {
        return Some((generator_parameters.0, generator_parameters.1));
    }

    let parameters = 'parameters: {
        match atomic {
            TAtomic::Iterable(iterable) => Some((iterable.get_key_type().clone(), iterable.get_value_type().clone())),
            TAtomic::Array(array_type) => Some(get_array_parameters(array_type, codebase, interner)),
            TAtomic::Object(object) => {
                let name = object.get_name()?;
                let traversable = interner.intern("traversable");

                let class_metadata = get_class_like(codebase, interner, name)?;
                if !is_instance_of(codebase, interner, &class_metadata.name, &traversable) {
                    break 'parameters None;
                }

                let traversable_metadata = get_class_like(codebase, interner, &traversable)?;
                let key_template = traversable_metadata.template_types.first().map(|(name, _)| name)?;
                let value_template = traversable_metadata.template_types.get(1).map(|(name, _)| name)?;

                let key_type = get_specialized_template_type(
                    codebase,
                    interner,
                    key_template,
                    &traversable,
                    class_metadata,
                    object.get_type_parameters(),
                )
                .unwrap_or_else(get_mixed);

                let value_type = get_specialized_template_type(
                    codebase,
                    interner,
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
            if let Some((key_type, value_type)) = get_iterable_parameters(intersection_type, codebase, interner) {
                return Some((key_type, value_type));
            }
        }
    }

    None
}

pub fn get_array_parameters(
    array_type: &TArray,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
) -> (TUnion, TUnion) {
    match array_type {
        TArray::Keyed(keyed_data) => {
            let mut key_types = vec![];
            let mut value_param;

            if let Some((key_param, value_p)) = &keyed_data.parameters {
                key_types.extend(key_param.types.clone());
                value_param = (**value_p).clone();
            } else {
                key_types.push(TAtomic::Never);
                value_param = get_never();
            }

            if let Some(known_items) = &keyed_data.known_items {
                for (key, (_, item_type)) in known_items {
                    key_types.push(key.to_atomic());
                    value_param = add_union_type(value_param, item_type, codebase, interner, false);
                }
            }

            let combined_key_types = combiner::combine(key_types, codebase, interner, false);
            let key_param_union = TUnion::new(combined_key_types);

            (key_param_union, value_param)
        }
        TArray::List(list_data) => {
            let mut key_types = vec![];
            let mut value_type = (*list_data.element_type).clone();

            if let Some(known_elements) = &list_data.known_elements {
                for (key_idx, (_, element_type)) in known_elements {
                    key_types.push(TAtomic::Scalar(TScalar::literal_int(*key_idx as i64)));

                    value_type = combine_union_types(element_type, &value_type, codebase, interner, false);
                }
            }

            if key_types.is_empty() || !value_type.is_never() {
                if value_type.is_never() {
                    key_types.push(TAtomic::Never);
                } else {
                    key_types.push(TAtomic::Scalar(TScalar::Integer(TInteger::non_negative())));
                }
            }

            let key_type = TUnion::new(combiner::combine(key_types, codebase, interner, false));

            (key_type, value_type)
        }
    }
}

pub fn get_iterable_value_parameter(
    atomic: &TAtomic,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
) -> Option<TUnion> {
    if let Some(generator_parameters) = atomic.get_generator_parameters(interner) {
        return Some(generator_parameters.1);
    }

    let parameter = match atomic {
        TAtomic::Iterable(iterable) => Some(iterable.get_value_type().clone()),
        TAtomic::Array(array_type) => Some(get_array_value_parameter(array_type, codebase, interner)),
        TAtomic::Object(object) => {
            let name = object.get_name()?;
            let traversable = interner.intern("traversable");

            let class_metadata = get_class_like(codebase, interner, name)?;
            if !is_instance_of(codebase, interner, &class_metadata.name, &traversable) {
                return None;
            }

            let traversable_metadata = get_class_like(codebase, interner, &traversable)?;
            let value_template = traversable_metadata.template_types.get(1).map(|(name, _)| name)?;

            get_specialized_template_type(
                codebase,
                interner,
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
            if let Some(value_param) = get_iterable_value_parameter(intersection_type, codebase, interner) {
                return Some(value_param);
            }
        }
    }

    None
}

pub fn get_array_value_parameter(
    array_type: &TArray,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
) -> TUnion {
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
                    value_param = combine_union_types(item_type, &value_param, codebase, interner, false);
                }
            }

            value_param
        }
        TArray::List(list_data) => {
            let mut value_param = (*list_data.element_type).clone();

            if let Some(known_elements) = &list_data.known_elements {
                for (_, element_type) in known_elements.values() {
                    value_param = combine_union_types(element_type, &value_param, codebase, interner, false);
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
    interner: &ThreadedInterner,
    template_name: &StringIdentifier,
    template_defining_class_id: &StringIdentifier,
    instantiated_class_metadata: &ClassLikeMetadata,
    instantiated_type_parameters: Option<&[TUnion]>,
) -> Option<TUnion> {
    let defining_class_metadata = get_class_like(codebase, interner, template_defining_class_id)?;

    if defining_class_metadata.name == instantiated_class_metadata.name {
        let index = instantiated_class_metadata.get_template_index_for_name(template_name)?;

        let Some(instantiated_type_parameters) = instantiated_type_parameters else {
            let type_map = instantiated_class_metadata.get_template_type(template_name)?;

            return type_map.first().map(|(_, constraint)| constraint).cloned();
        };

        return instantiated_type_parameters.get(index).cloned();
    }

    let defining_template_type = defining_class_metadata.get_template_type(template_name)?;
    let template_union = TUnion::new(
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

    let mut template_type = inferred_type_replacer::replace(&template_union, &template_result, codebase, interner);
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
            template_type = inferred_type_replacer::replace(&template_type, &template_result, codebase, interner);
        }
    }

    Some(template_type)
}
