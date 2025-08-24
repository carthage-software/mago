use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::r#enum::TEnum;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::union::TUnion;

pub mod r#enum;
pub mod named;

/// Represents a PHP object type, distinguishing between the generic `object`
/// and instances of specific named classes/interfaces/traits (which may include intersections).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub enum TObject {
    /// Represents the generic `object` type, accepting any object instance.
    Any,
    /// Represents an instance of a specific named class/interface/trait,
    /// potentially with generic parameters and intersection types (`&`).
    Named(TNamedObject),
    /// Represents a specific `enum` type (unit or backed).
    Enum(TEnum),
}

impl TObject {
    /// Creates a new `Object` representing the generic `object`.
    #[inline]
    pub const fn new_any() -> Self {
        TObject::Any
    }

    /// Creates a new `Object` representing a specific named object type (default flags).
    #[inline]
    pub fn new_named(name: Atom) -> Self {
        TObject::Named(TNamedObject::new(name))
    }

    /// Creates a new `Object` representing `$this` for a given class name.
    #[inline]
    pub fn new_named_this(name: Atom) -> Self {
        TObject::Named(TNamedObject::new_this(name))
    }

    /// Creates a new `TObject` representing an enum.
    #[inline]
    pub fn new_enum(name: Atom) -> Self {
        TObject::Enum(TEnum::new(name))
    }

    /// Creates a new `TObject` representing an enum case.
    #[inline]
    pub fn new_enum_case(name: Atom, case: Atom) -> Self {
        TObject::Enum(TEnum::new_case(name, case))
    }

    /// Checks if this represents the generic `object` type.
    #[inline]
    pub const fn is_any(&self) -> bool {
        matches!(self, TObject::Any)
    }

    /// Checks if this represents a specific named object type (including intersections).
    #[inline]
    pub const fn is_named(&self) -> bool {
        matches!(self, TObject::Named(_))
    }

    /// Checks if this represents a specific enum type.
    #[inline]
    pub const fn is_enum(&self) -> bool {
        matches!(self, TObject::Enum(_))
    }

    /// Checks if this represents an object that has a name.
    #[inline]
    pub const fn has_name(&self) -> bool {
        matches!(self, TObject::Named(_) | TObject::Enum(_))
    }

    /// Returns a reference to the `NamedObject` data if this is a `Named` variant.
    #[inline]
    pub const fn get_named_object_type(&self) -> Option<&TNamedObject> {
        if let TObject::Named(data) = self { Some(data) } else { None }
    }

    /// Returns a mutable reference to the `NamedObject` data if this is a `Named` variant.
    #[inline]
    pub const fn get_named_object_type_mut(&mut self) -> Option<&mut TNamedObject> {
        if let TObject::Named(data) = self { Some(data) } else { None }
    }

    /// Returns a reference to the `Enum` data if this is an `Enum` variant.
    #[inline]
    pub const fn get_enum_type(&self) -> Option<&TEnum> {
        if let TObject::Enum(data) = self { Some(data) } else { None }
    }

    /// Returns a mutable reference to the `Enum` data if this is an `Enum` variant.
    #[inline]
    pub const fn get_enum_type_mut(&mut self) -> Option<&mut TEnum> {
        if let TObject::Enum(data) = self { Some(data) } else { None }
    }

    /// Returns the primary name identifier if this is a `Named` or `Enum` variant.
    #[inline]
    pub const fn get_name(&self) -> Option<&Atom> {
        match self {
            TObject::Any => None,
            TObject::Enum(enum_object) => Some(&enum_object.name),
            TObject::Named(named_object) => Some(&named_object.name),
        }
    }

    /// Returns the type parameters of the named object if it has any.
    #[inline]
    pub fn get_type_parameters(&self) -> Option<&[TUnion]> {
        match self {
            TObject::Named(named_object) => named_object.get_type_parameters(),
            _ => None,
        }
    }

    /// Returns a slice of the additional intersection types (`&B&S`) if this is a `Named` object type.
    #[inline]
    pub fn get_intersection_types(&self) -> Option<&[TAtomic]> {
        match self {
            TObject::Named(named_object) => named_object.get_intersection_types(),
            _ => None,
        }
    }
}

impl TType for TObject {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        match self {
            TObject::Any => vec![],
            TObject::Enum(ttype) => ttype.get_child_nodes(),
            TObject::Named(ttype) => ttype.get_child_nodes(),
        }
    }

    fn can_be_intersected(&self) -> bool {
        matches!(self, TObject::Named(_))
    }

    fn get_intersection_types(&self) -> Option<&[TAtomic]> {
        match self {
            TObject::Named(named_object) => named_object.get_intersection_types(),
            _ => None,
        }
    }

    fn get_intersection_types_mut(&mut self) -> Option<&mut Vec<TAtomic>> {
        match self {
            TObject::Named(named_object) => named_object.get_intersection_types_mut(),
            _ => None,
        }
    }

    fn has_intersection_types(&self) -> bool {
        match self {
            TObject::Named(named_object) => named_object.has_intersection_types(),
            _ => false,
        }
    }

    fn add_intersection_type(&mut self, intersection_type: TAtomic) -> bool {
        match self {
            TObject::Named(named_object) => named_object.add_intersection_type(intersection_type),
            _ => false,
        }
    }

    fn needs_population(&self) -> bool {
        match self {
            TObject::Any => false,
            TObject::Enum(enum_object) => enum_object.needs_population(),
            TObject::Named(named_object) => named_object.needs_population(),
        }
    }

    fn is_expandable(&self) -> bool {
        match self {
            TObject::Any => false,
            TObject::Enum(enum_object) => enum_object.is_expandable(),
            TObject::Named(named_object) => named_object.is_expandable(),
        }
    }

    fn get_id(&self) -> Atom {
        match self {
            TObject::Any => atom("object"),
            TObject::Enum(enum_object) => enum_object.get_id(),
            TObject::Named(named_object) => named_object.get_id(),
        }
    }
}
