use std::borrow::Cow;
use std::hash::Hash;
use std::hash::Hasher;

use derivative::Derivative;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;

use crate::data_flow::node::DataFlowNode;
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
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::atomic::scalar::string::TStringLiteral;
use crate::ttype::get_arraykey;
use crate::ttype::get_int;
use crate::ttype::get_mixed;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Derivative, PartialOrd, Ord)]
pub struct TUnion {
    pub types: Vec<TAtomic>,
    pub parent_nodes: Vec<DataFlowNode>,
    pub had_template: bool,
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
        for t in &self.types {
            t.hash(state);
        }
    }
}

impl TUnion {
    pub fn new(mut types: Vec<TAtomic>) -> TUnion {
        if types.is_empty() {
            types.push(TAtomic::Never);
        }

        TUnion {
            types,
            parent_nodes: vec![],
            had_template: false,
            reference_free: false,
            possibly_undefined_from_try: false,
            possibly_undefined: false,
            ignore_falsable_issues: false,
            ignore_nullable_issues: false,
            from_template_default: false,
            populated: false,
        }
    }

    pub fn set_possibly_undefined(&mut self, possibly_undefined: bool, from_try: Option<bool>) {
        let from_try = from_try.unwrap_or(self.possibly_undefined_from_try);

        self.possibly_undefined = possibly_undefined;
        self.possibly_undefined_from_try = from_try;
    }

    /// Creates a new TUnion with the same properties as the original, but with a new set of types.
    pub fn clone_with_types(&self, types: Vec<TAtomic>) -> TUnion {
        TUnion {
            types,
            parent_nodes: self.parent_nodes.clone(),
            had_template: self.had_template,
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
            types: self.get_non_nullable_types(),
            parent_nodes: self.parent_nodes.clone(),
            had_template: self.had_template,
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
            types: self.get_truthy_types(),
            parent_nodes: self.parent_nodes.clone(),
            had_template: self.had_template,
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

    pub fn as_nullable(self) -> TUnion {
        TUnion {
            types: {
                let mut types = self.types;
                if !types.iter().any(|t| matches!(t, TAtomic::Null)) {
                    types.push(TAtomic::Null);
                }

                types
            },
            parent_nodes: self.parent_nodes,
            had_template: self.had_template,
            reference_free: self.reference_free,
            possibly_undefined_from_try: self.possibly_undefined_from_try,
            possibly_undefined: self.possibly_undefined,
            ignore_falsable_issues: self.ignore_falsable_issues,
            ignore_nullable_issues: self.ignore_nullable_issues,
            from_template_default: self.from_template_default,
            populated: self.populated,
        }
    }

    pub fn remove_type(&mut self, bad_type: &TAtomic) {
        self.types.retain(|t| t != bad_type);
    }

    pub fn is_int(&self) -> bool {
        for atomic in &self.types {
            if !atomic.is_int() {
                return false;
            }
        }

        true
    }

    pub fn has_int(&self) -> bool {
        for atomic in &self.types {
            if atomic.is_int() {
                return true;
            }
        }

        false
    }

    pub fn has_float(&self) -> bool {
        for atomic in &self.types {
            if atomic.is_float() {
                return true;
            }
        }

        false
    }

    pub fn is_array_key(&self) -> bool {
        for atomic in &self.types {
            if atomic.is_array_key() {
                continue;
            }

            return false;
        }

        true
    }

    pub fn is_any_string(&self) -> bool {
        for atomic in &self.types {
            if !atomic.is_any_string() {
                return false;
            }
        }

        true
    }

    pub fn is_string(&self) -> bool {
        self.types.iter().all(|t| t.is_string()) && !self.types.is_empty()
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
        self.types.iter().all(|t| t.is_never()) && !self.types.is_empty()
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

    pub fn is_any(&self) -> bool {
        self.types.len() == 1 && matches!(self.types[0], TAtomic::Mixed(mixed) if mixed.is_any())
    }

    pub fn is_numeric(&self) -> bool {
        self.types.iter().all(|t| t.is_numeric())
    }

    pub fn is_number(&self) -> bool {
        self.types.iter().all(|t| t.is_number())
    }

    pub fn is_mixed(&self) -> bool {
        self.types.iter().all(|t| matches!(t, TAtomic::Mixed(_))) && !self.types.is_empty()
    }

    pub fn is_mixed_template(&self) -> bool {
        self.types.iter().all(|t| t.is_templated_as_mixed(&mut false)) && !self.types.is_empty()
    }

    pub fn has_mixed(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Mixed(_))) && !self.types.is_empty()
    }

    pub fn has_mixed_template(&self) -> bool {
        self.types.iter().any(|t| t.is_templated_as_mixed(&mut false)) && !self.types.is_empty()
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

    pub fn is_mixed_with_any(&self, has_any: &mut bool) -> bool {
        if self.types.len() != 1 {
            return false;
        }

        match &self.types[0] {
            &TAtomic::Mixed(mixed) => {
                *has_any = mixed.is_any();
                true
            }
            _ => false,
        }
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
        for atomic in &self.types {
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
        for atomic in &self.types {
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
        for atomic in &self.types {
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

    pub fn is_generator(&self, interner: &ThreadedInterner) -> bool {
        for atomic in &self.types {
            if atomic.is_generator(interner) {
                continue;
            }

            return false;
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

    pub fn has_scalar(&self) -> bool {
        self.types.iter().any(|atomic| atomic.is_generic_scalar())
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

    pub fn has_traversable(&self, codebase: &CodebaseMetadata, interner: &ThreadedInterner) -> bool {
        self.types.iter().any(|atomic| atomic.is_traversable(codebase, interner)) && !self.types.is_empty()
    }

    pub fn has_array_key_like(&self) -> bool {
        self.types.iter().any(|atomic| atomic.is_array_key() || atomic.is_int() || atomic.is_string())
    }

    pub fn has_num(&self) -> bool {
        self.types
            .iter()
            .any(|atomic| atomic.is_numeric() || atomic.is_number() || atomic.is_int() || atomic.is_float())
            && !self.types.is_empty()
    }

    pub fn is_always_truthy(&self) -> bool {
        self.types.iter().all(|atomic| atomic.is_truthy()) && !self.types.is_empty()
    }

    pub fn is_always_falsy(&self) -> bool {
        self.types.iter().all(|atomic| atomic.is_falsy()) && !self.types.is_empty()
    }

    pub fn is_literal_of(&self, other: &TUnion) -> bool {
        let other_atomic_type = other.types.first().unwrap();

        match other_atomic_type {
            TAtomic::Scalar(TScalar::String(_)) => {
                for self_atomic_type in &self.types {
                    if self_atomic_type.is_string_of_literal_origin() {
                        continue;
                    }

                    return false;
                }

                true
            }
            TAtomic::Scalar(TScalar::Integer(_)) => {
                for self_atomic_type in &self.types {
                    if self_atomic_type.is_literal_int() {
                        continue;
                    }

                    return false;
                }

                true
            }
            TAtomic::Scalar(TScalar::Float(_)) => {
                for self_atomic_type in &self.types {
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
    pub fn has_object_type(&self) -> bool {
        self.types.iter().any(|t| matches!(t, TAtomic::Object(_)))
    }

    /// Return a vector of pairs containing the enum name, and their case name
    /// if specified.
    pub fn get_enum_cases(&self) -> Vec<(StringIdentifier, Option<StringIdentifier>)> {
        self.types
            .iter()
            .filter_map(|t| match t {
                TAtomic::Object(TObject::Enum(enum_object)) => Some((enum_object.name, enum_object.case)),
                _ => None,
            })
            .collect()
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

    pub fn get_single_class_string_value(&self) -> Option<StringIdentifier> {
        if self.is_single() { self.get_single().get_class_string_value() } else { None }
    }

    pub fn get_single_array_key(&self) -> Option<ArrayKey> {
        if self.is_single() { self.get_single().to_array_key() } else { None }
    }

    pub fn get_single_key_of_array_like(self) -> Option<TUnion> {
        if !self.is_single() {
            return None;
        }

        match self.get_single_owned() {
            TAtomic::Array(array) => match array {
                TArray::List(_) => Some(get_int()),
                TArray::Keyed(keyed_array) => match keyed_array.parameters {
                    Some((k, _)) => Some(*k),
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

    pub fn get_literal_string_values(&self) -> Vec<Option<String>> {
        self.get_literal_strings()
            .into_iter()
            .map(|atom| match atom {
                TAtomic::Scalar(TScalar::String(TString { literal: Some(TStringLiteral::Value(value)), .. })) => {
                    Some(value.clone())
                }
                _ => None,
            })
            .collect()
    }

    pub fn has_literal_value(&self) -> bool {
        self.types.iter().any(|atomic| match atomic {
            TAtomic::Scalar(scalar) => scalar.is_literal_value(),
            _ => false,
        })
    }

    pub fn needs_population(&self) -> bool {
        !self.populated || self.types.iter().any(|v| v.needs_population())
    }
}

impl TType for TUnion {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        self.types.iter().map(TypeRef::Atomic).collect()
    }

    fn get_id(&self, interner: Option<&ThreadedInterner>) -> String {
        let mut types = self.types.clone();
        let len = types.len();

        types.sort();
        types
            .iter()
            .map(|atomic| {
                let id = atomic.get_id(interner);
                if atomic.has_intersection_types() && len > 1 { format!("({id})") } else { id }
            })
            .join("|")
    }
}

impl PartialEq for TUnion {
    fn eq(&self, other: &TUnion) -> bool {
        let len = self.types.len();

        if len != other.types.len() {
            return false;
        }

        if len == 0 {
            if self.types[0] != other.types[0] {
                return false;
            }
        } else {
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
        }

        self.parent_nodes == other.parent_nodes
    }
}

pub fn populate_union_type(
    unpopulated_union: &mut TUnion,
    codebase_symbols: &Symbols,
    interner: &ThreadedInterner,
    reference_source: Option<&ReferenceSource>,
    symbol_references: &mut SymbolReferences,
    force: bool,
) {
    if unpopulated_union.populated && !force {
        return;
    }

    unpopulated_union.populated = true;

    for unpopulated_atomic in &mut unpopulated_union.types {
        match unpopulated_atomic {
            TAtomic::Scalar(TScalar::ClassLikeString(
                TClassLikeString::Generic { constraint, .. } | TClassLikeString::OfType { constraint, .. },
            )) => {
                let mut new_constraint = (**constraint).clone();

                populate_atomic_type(
                    &mut new_constraint,
                    codebase_symbols,
                    interner,
                    reference_source,
                    symbol_references,
                    force,
                );

                *constraint = Box::new(new_constraint);
            }
            _ => {
                populate_atomic_type(
                    unpopulated_atomic,
                    codebase_symbols,
                    interner,
                    reference_source,
                    symbol_references,
                    force,
                );
            }
        }
    }
}
