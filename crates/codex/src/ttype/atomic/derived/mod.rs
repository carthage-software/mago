use mago_atom::Atom;
use serde::Deserialize;
use serde::Serialize;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::derived::index_access::TIndexAccess;
use crate::ttype::atomic::derived::int_mask::TIntMask;
use crate::ttype::atomic::derived::int_mask_of::TIntMaskOf;
use crate::ttype::atomic::derived::key_of::TKeyOf;
use crate::ttype::atomic::derived::properties_of::TPropertiesOf;
use crate::ttype::atomic::derived::value_of::TValueOf;
use crate::ttype::union::TUnion;

pub mod index_access;
pub mod int_mask;
pub mod int_mask_of;
pub mod key_of;
pub mod properties_of;
pub mod value_of;

/// Represents derived/utility types that extract information from other types.
///
/// These types are used for introspection and manipulation of existing types:
///
/// - `key-of<T>`: Extracts the keys of an array-like type
/// - `value-of<T>`: Extracts the values of an array-like or enum type
/// - `properties-of<T>`: Extracts object properties, optionally filtered by visibility
/// - `T[K]`: Indexed access type that resolves to the type at index K of type T
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub enum TDerived {
    /// Represents `key-of<T>` utility type
    KeyOf(TKeyOf),
    /// Represents `value-of<T>` utility type
    ValueOf(TValueOf),
    /// Represents `int-mask<1, 2, 4>` utility type
    IntMask(TIntMask),
    /// Represents `int-mask-of<Foo::*>` utility type
    IntMaskOf(TIntMaskOf),
    /// Represents `properties-of<T>` utility type (including visibility-filtered variants)
    PropertiesOf(TPropertiesOf),
    /// Represents `T[K]` indexed access type
    IndexAccess(TIndexAccess),
}

impl TDerived {
    /// Returns the target type that this derived type operates on.
    ///
    /// Note: For `IntMask`, this returns `None` as it has multiple value types, not a single target.
    /// For `IntMaskOf`, this returns the target type reference.
    #[inline]
    #[must_use]
    pub fn get_target_type(&self) -> Option<&TUnion> {
        match self {
            TDerived::KeyOf(key_of) => Some(key_of.get_target_type()),
            TDerived::ValueOf(value_of) => Some(value_of.get_target_type()),
            TDerived::IntMask(_) => None,
            TDerived::IntMaskOf(int_mask_of) => Some(int_mask_of.get_target_type()),
            TDerived::PropertiesOf(properties_of) => Some(properties_of.get_target_type()),
            TDerived::IndexAccess(index_access) => Some(index_access.get_target_type()),
        }
    }

    /// Returns a mutable reference to the target type that this derived type operates on.
    ///
    /// Note: For `IntMask`, this returns `None` as it has multiple value types, not a single target.
    /// For `IntMaskOf`, this returns the target type reference.
    #[inline]
    pub fn get_target_type_mut(&mut self) -> Option<&mut TUnion> {
        match self {
            TDerived::KeyOf(key_of) => Some(key_of.get_target_type_mut()),
            TDerived::ValueOf(value_of) => Some(value_of.get_target_type_mut()),
            TDerived::IntMask(_) => None,
            TDerived::IntMaskOf(int_mask_of) => Some(int_mask_of.get_target_type_mut()),
            TDerived::PropertiesOf(properties_of) => Some(properties_of.get_target_type_mut()),
            TDerived::IndexAccess(index_access) => Some(index_access.get_target_type_mut()),
        }
    }
}

impl TType for TDerived {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        match self {
            TDerived::KeyOf(ttype) => ttype.get_child_nodes(),
            TDerived::ValueOf(ttype) => ttype.get_child_nodes(),
            TDerived::IntMask(ttype) => ttype.get_child_nodes(),
            TDerived::IntMaskOf(ttype) => ttype.get_child_nodes(),
            TDerived::PropertiesOf(ttype) => ttype.get_child_nodes(),
            TDerived::IndexAccess(ttype) => ttype.get_child_nodes(),
        }
    }

    fn needs_population(&self) -> bool {
        match self {
            TDerived::KeyOf(ttype) => ttype.needs_population(),
            TDerived::ValueOf(ttype) => ttype.needs_population(),
            TDerived::IntMask(ttype) => ttype.needs_population(),
            TDerived::IntMaskOf(ttype) => ttype.needs_population(),
            TDerived::PropertiesOf(ttype) => ttype.needs_population(),
            TDerived::IndexAccess(ttype) => ttype.needs_population(),
        }
    }

    fn is_expandable(&self) -> bool {
        match self {
            TDerived::KeyOf(ttype) => ttype.is_expandable(),
            TDerived::ValueOf(ttype) => ttype.is_expandable(),
            TDerived::IntMask(ttype) => ttype.is_expandable(),
            TDerived::IntMaskOf(ttype) => ttype.is_expandable(),
            TDerived::PropertiesOf(ttype) => ttype.is_expandable(),
            TDerived::IndexAccess(ttype) => ttype.is_expandable(),
        }
    }

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Atom {
        match self {
            TDerived::KeyOf(key_of) => key_of.get_id(),
            TDerived::ValueOf(value_of) => value_of.get_id(),
            TDerived::IntMask(int_mask) => int_mask.get_id(),
            TDerived::IntMaskOf(int_mask_of) => int_mask_of.get_id(),
            TDerived::PropertiesOf(properties_of) => properties_of.get_id(),
            TDerived::IndexAccess(index_access) => index_access.get_id(),
        }
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Atom {
        self.get_id()
    }
}
