use std::collections::BTreeMap;

use mago_word::Word;
use mago_word::word;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::r#enum::TEnum;
use crate::ttype::atomic::object::has_method::TObjectHasMethod;
use crate::ttype::atomic::object::has_property::TObjectHasProperty;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::object::with_properties::TObjectWithProperties;
use crate::ttype::union::TUnion;

macro_rules! has_member_ttype_impl {
    ($member_type:ident, $member_field:ident, $id_prefix:literal) => {
        impl TType for $member_type {
            fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
                self.intersection_types
                    .as_ref()
                    .map(|types| types.iter().map(TypeRef::Atomic).collect())
                    .unwrap_or_default()
            }

            fn can_be_intersected(&self) -> bool {
                true
            }

            fn get_intersection_types(&self) -> Option<&[TAtomic]> {
                self.intersection_types.as_deref()
            }

            fn get_intersection_types_mut(&mut self) -> Option<&mut Vec<TAtomic>> {
                self.intersection_types.as_mut()
            }

            fn has_intersection_types(&self) -> bool {
                self.intersection_types.as_ref().is_some_and(|v| !v.is_empty())
            }

            fn add_intersection_type(&mut self, intersection_type: TAtomic) -> bool {
                if let Some(intersection_types) = self.intersection_types.as_mut() {
                    intersection_types.push(intersection_type);
                } else {
                    self.intersection_types = Some(vec![intersection_type]);
                }

                true
            }

            fn get_id(&self) -> Word {
                let mut result = concat_word!($id_prefix, self.$member_field, b"'>");

                if let Some(intersection_types) = self.get_intersection_types() {
                    result = append_intersection_ids(result, intersection_types, None);
                }

                result
            }

            fn get_pretty_id_with_indent(&self, indent: usize) -> Word {
                let mut result = concat_word!($id_prefix, self.$member_field, b"'>");

                if let Some(intersection_types) = self.get_intersection_types() {
                    result = append_intersection_ids(result, intersection_types, Some(indent));
                }

                result
            }
        }
    };
}

pub mod r#enum;
pub mod has_method;
pub mod has_property;
pub mod named;
pub mod with_properties;

/// Represents a PHP object type, distinguishing between the generic `object`
/// and instances of specific named classes/interfaces/traits (which may include intersections).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TObject {
    /// Represents the generic `object` type, accepting any object instance.
    Any,
    /// Represents an instance of a specific named class/interface/trait,
    /// potentially with generic parameters and intersection types (`&`).
    Named(TNamedObject),
    /// Represents a specific `enum` type (unit or backed).
    Enum(TEnum),
    /// Represents an object documented via `object{...}` doc comment
    WithProperties(TObjectWithProperties),
    /// Represents an object with a known method from `method_exists()` check.
    HasMethod(TObjectHasMethod),
    /// Represents an object with a known property from `property_exists()` check.
    HasProperty(TObjectHasProperty),
}

impl TObject {
    /// Creates a new `Object` representing an object with specific known properties.
    ///
    /// The `sealed` flag indicates whether the object is sealed (no additional properties will exist beyond those known).
    ///
    /// The `known_properties` map defines specific types for certain keys (`Word`), where the `bool` indicates if the property is optional.
    #[inline]
    #[must_use]
    pub const fn new_with_properties(sealed: bool, known_properties: BTreeMap<Word, (bool, TUnion)>) -> Self {
        TObject::WithProperties(TObjectWithProperties { known_properties, sealed })
    }

    /// Creates a new `Object` representing a specific named object type (default flags).
    #[inline]
    #[must_use]
    pub fn new_named(name: Word) -> Self {
        TObject::Named(TNamedObject::new(name))
    }

    /// Creates a new `TObject` representing an enum.
    #[inline]
    #[must_use]
    pub fn new_enum(name: Word) -> Self {
        TObject::Enum(TEnum::new(name))
    }

    /// Creates a new `TObject` representing an enum case.
    #[inline]
    #[must_use]
    pub fn new_enum_case(name: Word, case: Word) -> Self {
        TObject::Enum(TEnum::new_case(name, case))
    }

    /// Creates a new `TObject` representing an object with a known method.
    #[inline]
    #[must_use]
    pub const fn new_has_method(method: Word) -> Self {
        TObject::HasMethod(TObjectHasMethod::new(method))
    }

    /// Creates a new `TObject` representing an object with a known property.
    #[inline]
    #[must_use]
    pub const fn new_has_property(property: Word) -> Self {
        TObject::HasProperty(TObjectHasProperty::new(property))
    }

    /// Checks if this represents a specific named object type (including intersections).
    #[inline]
    #[must_use]
    pub const fn is_named(&self) -> bool {
        matches!(self, TObject::Named(_))
    }

    /// Checks if this represents a specific enum type.
    #[inline]
    #[must_use]
    pub const fn is_enum(&self) -> bool {
        matches!(self, TObject::Enum(_))
    }

    /// Returns the primary name identifier if this is a `Named` or `Enum` variant.
    #[inline]
    #[must_use]
    pub const fn get_name(&self) -> Option<Word> {
        match self {
            TObject::Any | TObject::WithProperties(_) | TObject::HasMethod(_) | TObject::HasProperty(_) => None,
            TObject::Enum(enum_object) => Some(enum_object.name),
            TObject::Named(named_object) => Some(named_object.name),
        }
    }

    /// Returns the type parameters of the named object if it has any.
    #[inline]
    #[must_use]
    pub fn get_type_parameters(&self) -> Option<&[TUnion]> {
        match self {
            TObject::Named(named_object) => named_object.get_type_parameters(),
            _ => None,
        }
    }
}

impl TType for TObject {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        match self {
            TObject::Any => vec![],
            TObject::HasMethod(has_method) => has_method.get_child_nodes(),
            TObject::HasProperty(has_property) => has_property.get_child_nodes(),
            TObject::Enum(ttype) => ttype.get_child_nodes(),
            TObject::Named(ttype) => ttype.get_child_nodes(),
            TObject::WithProperties(ttype) => ttype.get_child_nodes(),
        }
    }

    fn can_be_intersected(&self) -> bool {
        match self {
            TObject::Named(named_object) => named_object.can_be_intersected(),
            TObject::HasMethod(has_method) => has_method.can_be_intersected(),
            TObject::HasProperty(has_property) => has_property.can_be_intersected(),
            _ => false,
        }
    }

    fn get_intersection_types(&self) -> Option<&[TAtomic]> {
        match self {
            TObject::Named(named_object) => named_object.get_intersection_types(),
            TObject::HasMethod(has_method) => has_method.get_intersection_types(),
            TObject::HasProperty(has_property) => has_property.get_intersection_types(),
            _ => None,
        }
    }

    fn get_intersection_types_mut(&mut self) -> Option<&mut Vec<TAtomic>> {
        match self {
            TObject::Named(named_object) => named_object.get_intersection_types_mut(),
            TObject::HasMethod(has_method) => has_method.get_intersection_types_mut(),
            TObject::HasProperty(has_property) => has_property.get_intersection_types_mut(),
            _ => None,
        }
    }

    fn has_intersection_types(&self) -> bool {
        match self {
            TObject::Named(named_object) => named_object.has_intersection_types(),
            TObject::HasMethod(has_method) => has_method.has_intersection_types(),
            TObject::HasProperty(has_property) => has_property.has_intersection_types(),
            _ => false,
        }
    }

    fn add_intersection_type(&mut self, intersection_type: TAtomic) -> bool {
        match self {
            TObject::Named(named_object) => named_object.add_intersection_type(intersection_type),
            TObject::HasMethod(has_method) => has_method.add_intersection_type(intersection_type),
            TObject::HasProperty(has_property) => has_property.add_intersection_type(intersection_type),
            _ => false,
        }
    }

    fn needs_population(&self) -> bool {
        match self {
            TObject::Any => false,
            TObject::HasMethod(has_method) => has_method.needs_population(),
            TObject::HasProperty(has_property) => has_property.needs_population(),
            TObject::Enum(enum_object) => enum_object.needs_population(),
            TObject::Named(named_object) => named_object.needs_population(),
            TObject::WithProperties(shaped_object) => shaped_object.needs_population(),
        }
    }

    fn is_expandable(&self) -> bool {
        match self {
            TObject::Any => false,
            TObject::HasMethod(has_method) => has_method.is_expandable(),
            TObject::HasProperty(has_property) => has_property.is_expandable(),
            TObject::Enum(enum_object) => enum_object.is_expandable(),
            TObject::Named(named_object) => named_object.is_expandable(),
            TObject::WithProperties(shaped_object) => shaped_object.is_expandable(),
        }
    }

    fn is_complex(&self) -> bool {
        match self {
            TObject::Any => false,
            TObject::HasMethod(has_method) => has_method.is_complex(),
            TObject::HasProperty(has_property) => has_property.is_complex(),
            TObject::Enum(enum_object) => enum_object.is_complex(),
            TObject::Named(named_object) => named_object.is_complex(),
            TObject::WithProperties(shaped_object) => shaped_object.is_complex(),
        }
    }

    fn get_id(&self) -> Word {
        match self {
            TObject::Any => word("object"),
            TObject::HasMethod(has_method) => has_method.get_id(),
            TObject::HasProperty(has_property) => has_property.get_id(),
            TObject::Enum(enum_object) => enum_object.get_id(),
            TObject::Named(named_object) => named_object.get_id(),
            TObject::WithProperties(shaped_object) => shaped_object.get_id(),
        }
    }

    fn get_pretty_id_with_indent(&self, indent: usize) -> Word {
        match self {
            TObject::Any => word("object"),
            TObject::HasMethod(has_method) => has_method.get_pretty_id_with_indent(indent),
            TObject::HasProperty(has_property) => has_property.get_pretty_id_with_indent(indent),
            TObject::Enum(enum_object) => enum_object.get_pretty_id_with_indent(indent),
            TObject::Named(named_object) => named_object.get_pretty_id_with_indent(indent),
            TObject::WithProperties(shaped_object) => shaped_object.get_pretty_id_with_indent(indent),
        }
    }
}
