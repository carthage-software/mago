use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;

use crate::is_instance_of;
use crate::metadata::CodebaseMetadata;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::symbol::SymbolKind;
use crate::symbol::Symbols;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::conditional::TConditional;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::iterable::TIterable;
use crate::ttype::atomic::mixed::TMixed;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::r#enum::TEnum;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::atomic::resource::TResource;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::atomic::scalar::string::TStringLiteral;
use crate::ttype::get_mixed;
use crate::ttype::union::TUnion;
use crate::ttype::union::populate_union_type;

pub mod array;
pub mod callable;
pub mod conditional;
pub mod derived;
pub mod generic;
pub mod iterable;
pub mod mixed;
pub mod object;
pub mod reference;
pub mod resource;
pub mod scalar;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub enum TAtomic {
    Scalar(TScalar),
    Callable(TCallable),
    Mixed(TMixed),
    Object(TObject),
    Array(TArray),
    Iterable(TIterable),
    Resource(TResource),
    Reference(TReference),
    GenericParameter(TGenericParameter),
    Variable(StringIdentifier),
    Conditional(TConditional),
    Derived(TDerived),
    Never,
    Null,
    Void,
    Placeholder,
}

impl TAtomic {
    pub fn is_numeric(&self) -> bool {
        match self {
            TAtomic::Scalar(scalar) => scalar.is_numeric(),
            TAtomic::GenericParameter(parameter) => parameter.constraint.is_numeric(),
            _ => false,
        }
    }

    pub fn is_int_or_float(&self) -> bool {
        match self {
            TAtomic::Scalar(scalar) => scalar.is_int_or_float(),
            TAtomic::GenericParameter(parameter) => parameter.constraint.is_int_or_float(),
            _ => false,
        }
    }

    pub fn is_num(&self) -> bool {
        match self {
            TAtomic::Scalar(scalar) => scalar.is_num(),
            _ => false,
        }
    }

    pub const fn is_mixed(&self) -> bool {
        matches!(self, TAtomic::Mixed(_))
    }

    pub const fn is_any(&self) -> bool {
        matches!(self, TAtomic::Mixed(mixed) if mixed.is_any())
    }

    pub const fn is_mixed_isset_from_loop(&self) -> bool {
        matches!(self, TAtomic::Mixed(mixed) if mixed.is_isset_from_loop())
    }

    pub const fn is_never(&self) -> bool {
        matches!(self, TAtomic::Never)
    }

    pub const fn is_mixed_with_any(&self, has_any: &mut bool) -> bool {
        match self {
            TAtomic::Mixed(mixed) => {
                *has_any = mixed.is_any();

                true
            }
            _ => false,
        }
    }

    pub fn is_templated_as_mixed(&self, has_any: &mut bool) -> bool {
        matches!(self, TAtomic::GenericParameter(parameter) if parameter.is_constrainted_as_mixed(has_any))
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, TAtomic::Object(TObject::Enum(TEnum { .. })))
    }

    pub fn is_enum_case(&self) -> bool {
        matches!(self, TAtomic::Object(TObject::Enum(TEnum { case: Some(_), .. })))
    }

    pub fn is_object_type(&self) -> bool {
        match self {
            TAtomic::Object(_) => true,
            TAtomic::Callable(callable) => callable.get_signature().is_none_or(|signature| signature.is_closure()),
            TAtomic::GenericParameter(parameter) => parameter.is_constrainted_as_objecty(),
            _ => false,
        }
    }

    pub fn is_this(&self) -> bool {
        matches!(self, TAtomic::Object(TObject::Named(named_object)) if named_object.is_this())
    }

    pub fn get_object_or_enum_name(&self) -> Option<StringIdentifier> {
        match self {
            TAtomic::Object(object) => match object {
                TObject::Named(named_object) => Some(named_object.get_name()),
                TObject::Enum(r#enum) => Some(r#enum.get_name()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn is_stdclass(&self, interner: &ThreadedInterner) -> bool {
        matches!(&self, TAtomic::Object(object) if {
            object.get_name().is_some_and(|name| interner.lookup(name).eq_ignore_ascii_case("stdClass"))
        })
    }

    pub fn is_generator(&self, interner: &ThreadedInterner) -> bool {
        matches!(&self, TAtomic::Object(object) if {
            object.get_name().is_some_and(|name| interner.lookup(name).eq_ignore_ascii_case("Generator"))
        })
    }

    pub fn get_generator_parameters(&self, interner: &ThreadedInterner) -> Option<(TUnion, TUnion, TUnion, TUnion)> {
        let generator_parameters = 'parameters: {
            let TAtomic::Object(TObject::Named(named_object)) = self else {
                break 'parameters None;
            };

            let name_str = interner.lookup(named_object.get_name_ref());
            if !name_str.eq_ignore_ascii_case("Generator") {
                break 'parameters None;
            }

            let parameters = named_object.get_type_parameters().unwrap_or_default();
            match parameters.len() {
                0 => Some((get_mixed(), get_mixed(), get_mixed(), get_mixed())),
                1 => Some((get_mixed(), parameters[0].clone(), get_mixed(), get_mixed())),
                2 => Some((parameters[0].clone(), parameters[1].clone(), get_mixed(), get_mixed())),
                3 => Some((parameters[0].clone(), parameters[1].clone(), parameters[2].clone(), get_mixed())),
                4 => Some((parameters[0].clone(), parameters[1].clone(), parameters[2].clone(), parameters[3].clone())),
                _ => None,
            }
        };

        if let Some(parameters) = generator_parameters {
            return Some(parameters);
        }

        if let Some(intersection_types) = self.get_intersection_types() {
            for intersection_type in intersection_types {
                if let Some(parameters) = intersection_type.get_generator_parameters(interner) {
                    return Some(parameters);
                }
            }
        }

        None
    }

    pub fn is_templated_as_object(&self) -> bool {
        matches!(self, TAtomic::GenericParameter(parameter) if {
            parameter.constraint.is_objecty() && parameter.intersection_types.is_none()
        })
    }

    #[inline]
    pub const fn is_list(&self) -> bool {
        matches!(self, TAtomic::Array(array) if array.is_list())
    }

    pub fn get_list_element_type(&self) -> Option<&TUnion> {
        match self {
            TAtomic::Array(array) => array.get_list().map(|list| list.get_element_type()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_non_empty_list(&self) -> bool {
        matches!(self, TAtomic::Array(array) if array.get_list().is_some_and(|list| list.is_non_empty()))
    }

    #[inline]
    pub fn is_empty_array(&self) -> bool {
        matches!(self, TAtomic::Array(array) if array.is_empty())
    }

    #[inline]
    pub const fn is_keyed_array(&self) -> bool {
        matches!(self, TAtomic::Array(array) if array.is_keyed())
    }

    pub fn is_non_empty_keyed_array(&self) -> bool {
        matches!(self, TAtomic::Array(array) if array.get_keyed().is_some_and(|keyed_array| keyed_array.is_non_empty()))
    }

    #[inline]
    pub const fn is_array(&self) -> bool {
        matches!(self, TAtomic::Array(_))
    }

    #[inline]
    pub const fn is_iterable(&self) -> bool {
        matches!(self, TAtomic::Iterable(_))
    }

    #[inline]
    pub fn extends_or_implements(
        &self,
        codebase: &CodebaseMetadata,
        interner: &ThreadedInterner,
        interface: StringIdentifier,
    ) -> bool {
        let object = match self {
            TAtomic::Object(object) => object,
            TAtomic::GenericParameter(parameter) => {
                if let Some(intersection_types) = parameter.get_intersection_types() {
                    for intersection_type in intersection_types {
                        if intersection_type.extends_or_implements(codebase, interner, interface) {
                            return true;
                        }
                    }
                }

                for constraint_atomic in &parameter.constraint.types {
                    if constraint_atomic.extends_or_implements(codebase, interner, interface) {
                        return true;
                    }
                }

                return false;
            }
            TAtomic::Iterable(iterable) => {
                if let Some(intersection_types) = iterable.get_intersection_types() {
                    for intersection_type in intersection_types {
                        if intersection_type.extends_or_implements(codebase, interner, interface) {
                            return true;
                        }
                    }
                }

                return false;
            }
            _ => return false,
        };

        if let Some(object_name) = object.get_name() {
            if *object_name == interface {
                return true;
            }

            if is_instance_of(codebase, interner, object_name, &interface) {
                return true;
            }
        }

        if let Some(intersection_types) = object.get_intersection_types() {
            for intersection_type in intersection_types {
                if intersection_type.extends_or_implements(codebase, interner, interface) {
                    return true;
                }
            }
        }

        false
    }

    #[inline]
    pub fn is_countable(&self, codebase: &CodebaseMetadata, interner: &ThreadedInterner) -> bool {
        match self {
            TAtomic::Array(_) => true,
            _ => self.extends_or_implements(codebase, interner, interner.intern("Countable")),
        }
    }

    #[inline]
    pub fn could_be_countable(&self, codebase: &CodebaseMetadata, interner: &ThreadedInterner) -> bool {
        self.is_mixed() || self.is_any() || self.is_countable(codebase, interner)
    }

    #[inline]
    pub fn is_traversable(&self, codebase: &CodebaseMetadata, interner: &ThreadedInterner) -> bool {
        self.extends_or_implements(codebase, interner, interner.intern("Traversable"))
            || self.extends_or_implements(codebase, interner, interner.intern("Iterator"))
            || self.extends_or_implements(codebase, interner, interner.intern("IteratorAggregate"))
            || self.extends_or_implements(codebase, interner, interner.intern("Generator"))
    }

    #[inline]
    pub fn is_array_or_traversable(&self, codebase: &CodebaseMetadata, interner: &ThreadedInterner) -> bool {
        match self {
            TAtomic::Iterable(_) => true,
            TAtomic::Array(_) => true,
            _ => self.is_traversable(codebase, interner),
        }
    }

    #[inline]
    pub fn could_be_array_or_traversable(&self, codebase: &CodebaseMetadata, interner: &ThreadedInterner) -> bool {
        self.is_mixed() || self.is_any() || self.is_array_or_traversable(codebase, interner)
    }

    pub fn is_non_empty_array(&self) -> bool {
        matches!(self, TAtomic::Array(array) if array.is_non_empty())
    }

    pub fn to_array_key(&self) -> Option<ArrayKey> {
        match self {
            TAtomic::Scalar(TScalar::Integer(int)) => int.get_literal_value().map(ArrayKey::Integer),
            TAtomic::Scalar(TScalar::String(TString { literal: Some(TStringLiteral::Value(value)), .. })) => {
                Some(ArrayKey::String(value.clone()))
            }
            _ => None,
        }
    }

    pub fn get_array_key_type(&self) -> Option<TUnion> {
        match self {
            TAtomic::Array(array) => array.get_key_type(),
            _ => None,
        }
    }

    pub fn get_array_value_type(&self) -> Option<TUnion> {
        match self {
            TAtomic::Array(array) => array.get_value_type(),
            _ => None,
        }
    }

    #[inline]
    pub const fn is_generic_scalar(&self) -> bool {
        matches!(self, TAtomic::Scalar(TScalar::Generic))
    }

    #[inline]
    pub const fn is_some_scalar(&self) -> bool {
        matches!(self, TAtomic::Scalar(_))
    }

    #[inline]
    pub const fn is_boring_scalar(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_boring()
        )
    }

    #[inline]
    pub const fn is_any_string(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_any_string()
        )
    }

    #[inline]
    pub const fn is_string(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_string()
        )
    }

    #[inline]
    pub const fn is_string_of_literal_origin(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_literal_origin_string()
        )
    }

    #[inline]
    pub const fn is_non_empty_string(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_non_empty_string()
        )
    }

    #[inline]
    pub const fn is_known_literal_string(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_known_literal_string()
        )
    }

    pub const fn is_string_subtype(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_non_boring_string()
        )
    }

    #[inline]
    pub const fn is_array_key(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_array_key()
        )
    }

    #[inline]
    pub const fn is_int(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_int()
        )
    }

    #[inline]
    pub const fn is_literal_int(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_literal_int()
        )
    }

    #[inline]
    pub const fn is_float(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_float()
        )
    }

    #[inline]
    pub const fn is_literal_float(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_literal_float()
        )
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        matches!(self, TAtomic::Null)
    }

    #[inline]
    pub const fn is_void(&self) -> bool {
        matches!(self, TAtomic::Void)
    }

    #[inline]
    pub const fn is_bool(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_bool()
        )
    }

    #[inline]
    pub const fn is_general_bool(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_general_bool()
        )
    }

    #[inline]
    pub const fn is_general_string(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_general_string()
        )
    }

    #[inline]
    pub const fn is_true(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_true()
        )
    }

    #[inline]
    pub const fn is_false(&self) -> bool {
        matches!(
            self,
            TAtomic::Scalar(scalar) if scalar.is_false()
        )
    }

    #[inline]
    pub const fn is_resource(&self) -> bool {
        matches!(self, TAtomic::Resource(_))
    }

    #[inline]
    pub const fn is_closed_resource(&self) -> bool {
        matches!(self, TAtomic::Resource(resource) if resource.is_closed())
    }

    #[inline]
    pub const fn is_open_resource(&self) -> bool {
        matches!(self, TAtomic::Resource(resource) if resource.is_open())
    }

    #[inline]
    pub const fn is_literal(&self) -> bool {
        match self {
            TAtomic::Scalar(scalar) => scalar.is_literal_value(),
            TAtomic::Null => true,
            _ => false,
        }
    }

    #[inline]
    pub const fn is_callable(&self) -> bool {
        matches!(self, TAtomic::Callable(_))
    }

    #[inline]
    pub const fn is_conditional(&self) -> bool {
        matches!(self, TAtomic::Conditional(_))
    }

    #[inline]
    pub const fn is_generic_parameter(&self) -> bool {
        matches!(self, TAtomic::GenericParameter(_))
    }

    #[inline]
    pub const fn get_generic_parameter_name(&self) -> Option<StringIdentifier> {
        match self {
            TAtomic::GenericParameter(parameter) => Some(parameter.parameter_name),
            _ => None,
        }
    }

    /// Is this a type that could potentially be callable at runtime?
    #[inline]
    pub const fn can_be_callable(&self) -> bool {
        matches!(
            self,
            TAtomic::Callable(_)
                | TAtomic::Scalar(TScalar::String(_))
                | TAtomic::Array(TArray::List(_))
                | TAtomic::Object(TObject::Named(_))
        )
    }

    pub fn replace_template_constraint(&self, new_as_type: TUnion) -> TAtomic {
        if let Self::GenericParameter(parameter) = self {
            return TAtomic::GenericParameter(parameter.with_constraint(new_as_type));
        }

        panic!("replace_template_constraint called on non-generic parameter type");
    }

    pub fn get_non_empty_list(&self, known_count: Option<usize>) -> TAtomic {
        if let TAtomic::Array(TArray::List(list)) = self {
            return TAtomic::Array(TArray::List(list.clone_non_empty_with_count(known_count)));
        }

        panic!("get_non_empty_list called on non-list type");
    }

    pub fn make_non_empty_keyed_array(mut self) -> TAtomic {
        if let TAtomic::Array(TArray::Keyed(keyed_array)) = &mut self {
            keyed_array.non_empty = true;

            return self;
        }

        unreachable!("make_non_empty_keyed_array called on non-keyed array type");
    }

    pub fn is_truthy(&self) -> bool {
        match &self {
            TAtomic::Scalar(scalar) => scalar.is_truthy(),
            TAtomic::Array(array) => array.is_truthy(),
            TAtomic::Mixed(mixed) => mixed.is_truthy(),
            TAtomic::Object(_) | TAtomic::Callable(_) => true,
            _ => false,
        }
    }

    pub fn is_falsy(&self) -> bool {
        match &self {
            TAtomic::Scalar(scalar) if scalar.is_falsy() => true,
            TAtomic::Array(array) if array.is_falsy() => true,
            TAtomic::Mixed(mixed) if mixed.is_falsy() => true,
            TAtomic::Null => true,
            _ => false,
        }
    }

    pub fn is_array_accessible_with_string_key(&self) -> bool {
        matches!(self, TAtomic::Array(array) if array.is_keyed())
    }

    pub fn is_array_accessible_with_int_or_string_key(&self) -> bool {
        matches!(self, TAtomic::Array(_))
    }

    #[inline]
    pub fn needs_population(&self) -> bool {
        matches!(
            self,
            TAtomic::Array(_)
                | TAtomic::Iterable(_)
                | TAtomic::Callable(TCallable::Signature(_))
                | TAtomic::Object(TObject::Named(_))
                | TAtomic::Reference { .. }
                | TAtomic::GenericParameter(_)
                | TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic { .. }))
                | TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { .. }))
        )
    }

    pub fn clone_without_intersection_types(&self) -> TAtomic {
        let mut clone = self.clone();
        match &mut clone {
            TAtomic::Object(TObject::Named(named_object)) => {
                named_object.intersection_types = None;
            }
            TAtomic::GenericParameter(parameter) => {
                parameter.intersection_types = None;
            }
            TAtomic::Iterable(iterable) => {
                iterable.intersection_types = None;
            }
            TAtomic::Reference(TReference::Symbol { intersection_types, .. }) => {
                *intersection_types = None;
            }
            _ => {}
        }

        clone
    }

    pub fn remove_placeholders(&mut self, interner: &ThreadedInterner) {
        match self {
            TAtomic::Array(array) => {
                array.remove_placeholders();
            }
            TAtomic::Object(TObject::Named(named_object)) => {
                let name = named_object.get_name();
                if let Some(type_parameters) = named_object.get_type_parameters_mut() {
                    let name_str = interner.lookup(&name);
                    if name_str.eq_ignore_ascii_case("Traversable") {
                        let has_kv_pair = type_parameters.len() == 2;

                        if let Some(key_or_value_param) = type_parameters.get_mut(0)
                            && let TAtomic::Placeholder = key_or_value_param.get_single()
                        {
                            *key_or_value_param = if has_kv_pair {
                                TUnion::new(vec![TAtomic::Scalar(TScalar::ArrayKey)])
                            } else {
                                TUnion::new(vec![TAtomic::Mixed(TMixed::any())])
                            };
                        }

                        if has_kv_pair
                            && let Some(value_param) = type_parameters.get_mut(1)
                            && let TAtomic::Placeholder = value_param.get_single()
                        {
                            *value_param = TUnion::new(vec![TAtomic::Mixed(TMixed::any())]);
                        }
                    } else {
                        for type_param in type_parameters {
                            if let TAtomic::Placeholder = type_param.get_single() {
                                *type_param = TUnion::new(vec![TAtomic::Mixed(TMixed::any())]);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn get_literal_string_value(&self) -> Option<&str> {
        match self {
            TAtomic::Scalar(scalar) => scalar.get_known_literal_string_value(),
            _ => None,
        }
    }

    pub fn get_class_string_value(&self) -> Option<StringIdentifier> {
        match self {
            TAtomic::Scalar(scalar) => scalar.get_literal_class_string_value(),
            _ => None,
        }
    }

    pub fn get_integer(&self) -> Option<TInteger> {
        match self {
            TAtomic::Scalar(TScalar::Integer(integer)) => Some(*integer),
            _ => None,
        }
    }

    pub fn get_literal_int_value(&self) -> Option<i64> {
        match self {
            TAtomic::Scalar(scalar) => scalar.get_literal_int_value(),
            _ => None,
        }
    }

    pub fn get_maximum_int_value(&self) -> Option<i64> {
        match self {
            TAtomic::Scalar(scalar) => scalar.get_maximum_int_value(),
            _ => None,
        }
    }

    pub fn get_minimum_int_value(&self) -> Option<i64> {
        match self {
            TAtomic::Scalar(scalar) => scalar.get_minimum_int_value(),
            _ => None,
        }
    }

    pub fn get_literal_float_value(&self) -> Option<f64> {
        match self {
            TAtomic::Scalar(scalar) => scalar.get_literal_float_value(),
            _ => None,
        }
    }
}

impl TType for TAtomic {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        match self {
            TAtomic::Array(ttype) => ttype.get_child_nodes(),
            TAtomic::Callable(ttype) => ttype.get_child_nodes(),
            TAtomic::Conditional(ttype) => ttype.get_child_nodes(),
            TAtomic::Derived(ttype) => ttype.get_child_nodes(),
            TAtomic::GenericParameter(ttype) => ttype.get_child_nodes(),
            TAtomic::Iterable(ttype) => ttype.get_child_nodes(),
            TAtomic::Mixed(ttype) => ttype.get_child_nodes(),
            TAtomic::Object(ttype) => ttype.get_child_nodes(),
            TAtomic::Reference(ttype) => ttype.get_child_nodes(),
            TAtomic::Resource(ttype) => ttype.get_child_nodes(),
            TAtomic::Scalar(ttype) => ttype.get_child_nodes(),
            _ => vec![],
        }
    }

    fn can_be_intersected(&self) -> bool {
        match self {
            TAtomic::Object(ttype) => ttype.can_be_intersected(),
            TAtomic::Reference(ttype) => ttype.can_be_intersected(),
            TAtomic::GenericParameter(ttype) => ttype.can_be_intersected(),
            TAtomic::Iterable(ttype) => ttype.can_be_intersected(),
            TAtomic::Array(ttype) => ttype.can_be_intersected(),
            TAtomic::Callable(ttype) => ttype.can_be_intersected(),
            TAtomic::Mixed(ttype) => ttype.can_be_intersected(),
            TAtomic::Scalar(ttype) => ttype.can_be_intersected(),
            TAtomic::Resource(ttype) => ttype.can_be_intersected(),
            TAtomic::Conditional(ttype) => ttype.can_be_intersected(),
            TAtomic::Derived(ttype) => ttype.can_be_intersected(),
            _ => false,
        }
    }

    fn get_intersection_types(&self) -> Option<&[TAtomic]> {
        match self {
            TAtomic::Object(ttype) => ttype.get_intersection_types(),
            TAtomic::Reference(ttype) => ttype.get_intersection_types(),
            TAtomic::GenericParameter(ttype) => ttype.get_intersection_types(),
            TAtomic::Iterable(ttype) => ttype.get_intersection_types(),
            TAtomic::Array(ttype) => ttype.get_intersection_types(),
            TAtomic::Callable(ttype) => ttype.get_intersection_types(),
            TAtomic::Mixed(ttype) => ttype.get_intersection_types(),
            TAtomic::Scalar(ttype) => ttype.get_intersection_types(),
            TAtomic::Resource(ttype) => ttype.get_intersection_types(),
            TAtomic::Conditional(ttype) => ttype.get_intersection_types(),
            TAtomic::Derived(ttype) => ttype.get_intersection_types(),
            _ => None,
        }
    }

    fn get_intersection_types_mut(&mut self) -> Option<&mut Vec<TAtomic>> {
        match self {
            TAtomic::Object(ttype) => ttype.get_intersection_types_mut(),
            TAtomic::Reference(ttype) => ttype.get_intersection_types_mut(),
            TAtomic::GenericParameter(ttype) => ttype.get_intersection_types_mut(),
            TAtomic::Iterable(ttype) => ttype.get_intersection_types_mut(),
            TAtomic::Array(ttype) => ttype.get_intersection_types_mut(),
            TAtomic::Callable(ttype) => ttype.get_intersection_types_mut(),
            TAtomic::Mixed(ttype) => ttype.get_intersection_types_mut(),
            TAtomic::Scalar(ttype) => ttype.get_intersection_types_mut(),
            TAtomic::Resource(ttype) => ttype.get_intersection_types_mut(),
            TAtomic::Conditional(ttype) => ttype.get_intersection_types_mut(),
            TAtomic::Derived(ttype) => ttype.get_intersection_types_mut(),
            _ => None,
        }
    }

    fn has_intersection_types(&self) -> bool {
        match self {
            TAtomic::Object(ttype) => ttype.has_intersection_types(),
            TAtomic::Reference(ttype) => ttype.has_intersection_types(),
            TAtomic::GenericParameter(ttype) => ttype.has_intersection_types(),
            TAtomic::Iterable(ttype) => ttype.has_intersection_types(),
            TAtomic::Array(ttype) => ttype.has_intersection_types(),
            TAtomic::Callable(ttype) => ttype.has_intersection_types(),
            TAtomic::Mixed(ttype) => ttype.has_intersection_types(),
            TAtomic::Scalar(ttype) => ttype.has_intersection_types(),
            TAtomic::Resource(ttype) => ttype.has_intersection_types(),
            TAtomic::Conditional(ttype) => ttype.has_intersection_types(),
            TAtomic::Derived(ttype) => ttype.has_intersection_types(),
            _ => false,
        }
    }

    fn add_intersection_type(&mut self, intersection_type: TAtomic) -> bool {
        match self {
            TAtomic::Object(ttype) => ttype.add_intersection_type(intersection_type),
            TAtomic::Reference(ttype) => ttype.add_intersection_type(intersection_type),
            TAtomic::GenericParameter(ttype) => ttype.add_intersection_type(intersection_type),
            TAtomic::Iterable(ttype) => ttype.add_intersection_type(intersection_type),
            TAtomic::Array(ttype) => ttype.add_intersection_type(intersection_type),
            TAtomic::Callable(ttype) => ttype.add_intersection_type(intersection_type),
            TAtomic::Mixed(ttype) => ttype.add_intersection_type(intersection_type),
            TAtomic::Scalar(ttype) => ttype.add_intersection_type(intersection_type),
            TAtomic::Resource(ttype) => ttype.add_intersection_type(intersection_type),
            TAtomic::Conditional(ttype) => ttype.add_intersection_type(intersection_type),
            TAtomic::Derived(ttype) => ttype.add_intersection_type(intersection_type),
            _ => false,
        }
    }

    fn get_id(&self, interner: Option<&ThreadedInterner>) -> String {
        match self {
            TAtomic::Scalar(scalar) => scalar.get_id(interner),
            TAtomic::Array(array) => array.get_id(interner),
            TAtomic::Callable(callable) => callable.get_id(interner),
            TAtomic::Object(object) => object.get_id(interner),
            TAtomic::Reference(reference) => reference.get_id(interner),
            TAtomic::Mixed(mixed) => mixed.get_id(interner),
            TAtomic::Resource(resource) => resource.get_id(interner),
            TAtomic::Iterable(iterable) => iterable.get_id(interner),
            TAtomic::GenericParameter(parameter) => parameter.get_id(interner),
            TAtomic::Conditional(conditional) => conditional.get_id(interner),
            TAtomic::Derived(derived) => derived.get_id(interner),
            TAtomic::Variable(name) => {
                if let Some(interner) = interner {
                    interner.lookup(name).to_string()
                } else {
                    name.to_string()
                }
            }
            TAtomic::Never => "never".to_string(),
            TAtomic::Null => "null".to_string(),
            TAtomic::Void => "void".to_string(),
            TAtomic::Placeholder => "_".to_string(),
        }
    }
}

pub fn populate_atomic_type(
    unpopulated_atomic: &mut TAtomic,
    codebase_symbols: &Symbols,
    interner: &ThreadedInterner,
    reference_source: Option<&ReferenceSource>,
    symbol_references: &mut SymbolReferences,
    force: bool,
) {
    match unpopulated_atomic {
        TAtomic::Array(array) => match array {
            TArray::List(list) => {
                populate_union_type(
                    list.element_type.as_mut(),
                    codebase_symbols,
                    interner,
                    reference_source,
                    symbol_references,
                    force,
                );

                if let Some(known_elements) = list.known_elements.as_mut() {
                    for (_, element_type) in known_elements.values_mut() {
                        populate_union_type(
                            element_type,
                            codebase_symbols,
                            interner,
                            reference_source,
                            symbol_references,
                            force,
                        );
                    }
                }
            }
            TArray::Keyed(keyed_array) => {
                if let Some(known_items) = keyed_array.known_items.as_mut() {
                    for (_, item_type) in known_items.values_mut() {
                        populate_union_type(
                            item_type,
                            codebase_symbols,
                            interner,
                            reference_source,
                            symbol_references,
                            force,
                        );
                    }
                }

                if let Some(parameters) = &mut keyed_array.parameters {
                    populate_union_type(
                        parameters.0.as_mut(),
                        codebase_symbols,
                        interner,
                        reference_source,
                        symbol_references,
                        force,
                    );

                    populate_union_type(
                        parameters.1.as_mut(),
                        codebase_symbols,
                        interner,
                        reference_source,
                        symbol_references,
                        force,
                    );
                }
            }
        },
        TAtomic::Callable(TCallable::Signature(signature)) => {
            if let Some(return_type) = signature.get_return_type_mut() {
                populate_union_type(
                    return_type,
                    codebase_symbols,
                    interner,
                    reference_source,
                    symbol_references,
                    force,
                );
            }

            for param in signature.get_parameters_mut() {
                if let Some(param_type) = param.get_type_signature_mut() {
                    populate_union_type(
                        param_type,
                        codebase_symbols,
                        interner,
                        reference_source,
                        symbol_references,
                        force,
                    );
                }
            }
        }
        TAtomic::Object(TObject::Named(named_object)) => {
            let name = named_object.get_name();

            if !named_object.is_intersection()
                && !named_object.has_type_parameters()
                && codebase_symbols.contains_enum(&name)
            {
                *unpopulated_atomic = TAtomic::Object(TObject::new_enum(name));
            } else {
                if let Some(type_parameters) = named_object.get_type_parameters_mut() {
                    for parameter in type_parameters {
                        populate_union_type(
                            parameter,
                            codebase_symbols,
                            interner,
                            reference_source,
                            symbol_references,
                            force,
                        );
                    }
                }

                if let Some(intersection_types) = named_object.get_intersection_types_mut() {
                    for intersection_type in intersection_types {
                        populate_atomic_type(
                            intersection_type,
                            codebase_symbols,
                            interner,
                            reference_source,
                            symbol_references,
                            force,
                        );
                    }
                }
            }

            if let Some(reference_source) = reference_source {
                match reference_source {
                    ReferenceSource::Symbol(in_signature, a) => {
                        symbol_references.add_symbol_reference_to_symbol(*a, name, *in_signature)
                    }
                    ReferenceSource::ClassLikeMember(in_signature, a, b) => {
                        symbol_references.add_class_member_reference_to_symbol((*a, *b), name, *in_signature)
                    }
                }
            }
        }
        TAtomic::Iterable(iterable) => {
            populate_union_type(
                iterable.get_key_type_mut(),
                codebase_symbols,
                interner,
                reference_source,
                symbol_references,
                force,
            );

            populate_union_type(
                iterable.get_value_type_mut(),
                codebase_symbols,
                interner,
                reference_source,
                symbol_references,
                force,
            );

            if let Some(intersection_types) = iterable.get_intersection_types_mut() {
                for intersection_type in intersection_types {
                    populate_atomic_type(
                        intersection_type,
                        codebase_symbols,
                        interner,
                        reference_source,
                        symbol_references,
                        force,
                    );
                }
            }
        }
        TAtomic::Reference(reference) => match reference {
            TReference::Symbol { name, parameters, intersection_types } => {
                let lower_name = interner.lowered(name);

                if let Some(parameters) = parameters {
                    for parameter in parameters {
                        populate_union_type(
                            parameter,
                            codebase_symbols,
                            interner,
                            reference_source,
                            symbol_references,
                            force,
                        );
                    }
                }

                if let Some(reference_source) = reference_source {
                    match reference_source {
                        ReferenceSource::Symbol(in_signature, a) => {
                            symbol_references.add_symbol_reference_to_symbol(*a, *name, *in_signature)
                        }
                        ReferenceSource::ClassLikeMember(in_signature, a, b) => {
                            symbol_references.add_class_member_reference_to_symbol((*a, *b), *name, *in_signature)
                        }
                    }
                }

                if let Some(symbol_kind) = codebase_symbols.get_kind(&lower_name) {
                    match symbol_kind {
                        SymbolKind::Enum => {
                            *unpopulated_atomic = TAtomic::Object(TObject::new_enum(*name));
                        }
                        _ => {
                            let intersection_types = intersection_types.take().map(|intersection_types| {
                                intersection_types
                                    .into_iter()
                                    .map(|mut intersection_type| {
                                        populate_atomic_type(
                                            &mut intersection_type,
                                            codebase_symbols,
                                            interner,
                                            reference_source,
                                            symbol_references,
                                            force,
                                        );

                                        intersection_type
                                    })
                                    .collect::<Vec<_>>()
                            });

                            let mut named_object = TNamedObject::new(*name).with_type_parameters(parameters.clone());
                            if let Some(intersection_types) = intersection_types {
                                for intersection_type in intersection_types {
                                    named_object.add_intersection_type(intersection_type);
                                }
                            }

                            *unpopulated_atomic = TAtomic::Object(TObject::Named(named_object));
                        }
                    }
                }
            }
            TReference::Member { class_like_name, member_selector } => {
                if let TReferenceMemberSelector::Identifier(member_name) = member_selector
                    && let Some(reference_source) = reference_source
                {
                    match reference_source {
                        ReferenceSource::Symbol(in_signature, a) => symbol_references
                            .add_symbol_reference_to_class_member(*a, (*class_like_name, *member_name), *in_signature),
                        ReferenceSource::ClassLikeMember(in_signature, a, b) => symbol_references
                            .add_class_member_reference_to_class_member(
                                (*a, *b),
                                (*class_like_name, *member_name),
                                *in_signature,
                            ),
                    }
                }
            }
        },
        TAtomic::GenericParameter(TGenericParameter { constraint, intersection_types, .. }) => {
            populate_union_type(
                constraint.as_mut(),
                codebase_symbols,
                interner,
                reference_source,
                symbol_references,
                force,
            );

            if let Some(intersection_types) = intersection_types.as_mut() {
                for intersection_type in intersection_types {
                    populate_atomic_type(
                        intersection_type,
                        codebase_symbols,
                        interner,
                        reference_source,
                        symbol_references,
                        force,
                    );
                }
            }
        }
        TAtomic::Scalar(TScalar::ClassLikeString(
            TClassLikeString::OfType { constraint, .. } | TClassLikeString::Generic { constraint, .. },
        )) => {
            populate_atomic_type(
                constraint.as_mut(),
                codebase_symbols,
                interner,
                reference_source,
                symbol_references,
                force,
            );
        }
        TAtomic::Conditional(conditional) => {
            populate_union_type(
                conditional.get_subject_mut(),
                codebase_symbols,
                interner,
                reference_source,
                symbol_references,
                force,
            );

            populate_union_type(
                conditional.get_target_mut(),
                codebase_symbols,
                interner,
                reference_source,
                symbol_references,
                force,
            );

            populate_union_type(
                conditional.get_then_mut(),
                codebase_symbols,
                interner,
                reference_source,
                symbol_references,
                force,
            );

            populate_union_type(
                conditional.get_otherwise_mut(),
                codebase_symbols,
                interner,
                reference_source,
                symbol_references,
                force,
            );
        }
        TAtomic::Derived(derived) => {
            populate_atomic_type(
                derived.get_target_type_mut(),
                codebase_symbols,
                interner,
                reference_source,
                symbol_references,
                force,
            );
        }
        _ => {}
    }
}
