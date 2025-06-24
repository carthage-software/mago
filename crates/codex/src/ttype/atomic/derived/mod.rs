use serde::Deserialize;
use serde::Serialize;

use mago_interner::ThreadedInterner;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::derived::key_of::TKeyOf;
use crate::ttype::atomic::derived::properties_of::TPropertiesOf;
use crate::ttype::atomic::derived::value_of::TValueOf;

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub enum TDerived {
    /// Represents `key-of<T>` utility type
    KeyOf(TKeyOf),
    /// Represents `value-of<T>` utility type
    ValueOf(TValueOf),
    /// Represents `properties-of<T>` utility type (including visibility-filtered variants)
    PropertiesOf(TPropertiesOf),
}

impl TDerived {
    /// Returns the target type that this derived type operates on
    #[inline]
    pub const fn get_target_type(&self) -> &TAtomic {
        match self {
            TDerived::KeyOf(key_of) => key_of.get_target_type(),
            TDerived::ValueOf(value_of) => value_of.get_target_type(),
            TDerived::PropertiesOf(properties_of) => properties_of.get_target_type(),
        }
    }

    /// Returns a mutable reference to the target type that this derived type operates on
    #[inline]
    pub const fn get_target_type_mut(&mut self) -> &mut TAtomic {
        match self {
            TDerived::KeyOf(key_of) => key_of.get_target_type_mut(),
            TDerived::ValueOf(value_of) => value_of.get_target_type_mut(),
            TDerived::PropertiesOf(properties_of) => properties_of.get_target_type_mut(),
        }
    }
}

impl TType for TDerived {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        match self {
            TDerived::KeyOf(ttype) => ttype.get_child_nodes(),
            TDerived::ValueOf(ttype) => ttype.get_child_nodes(),
            TDerived::PropertiesOf(ttype) => ttype.get_child_nodes(),
        }
    }

    fn get_id(&self, interner: Option<&ThreadedInterner>) -> String {
        match self {
            TDerived::KeyOf(key_of) => key_of.get_id(interner),
            TDerived::ValueOf(value_of) => value_of.get_id(interner),
            TDerived::PropertiesOf(properties_of) => properties_of.get_id(interner),
        }
    }
}
