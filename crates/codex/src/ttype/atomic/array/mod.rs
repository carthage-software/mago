use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::get_arraykey;
use crate::ttype::get_int;
use crate::ttype::get_mixed;
use crate::ttype::union::TUnion;

pub mod key;
pub mod keyed;
pub mod list;

/// Represents the type of a PHP array, distinguishing between list-like and keyed/associative usage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub enum TArray {
    /// Represents an array used as a list (sequential, zero-based integer keys). `list<T>`.
    List(TList),
    /// Represents an array used as a map (string keys or non-standard integer keys). `array<Tk, Tv>`.
    Keyed(TKeyedArray),
}

impl TArray {
    /// Creates a new `ArrayType::List` with the given element type.
    #[inline]
    pub fn new_list(element_type: Box<TUnion>) -> Self {
        Self::List(TList::new(element_type))
    }

    /// Creates a new `ArrayType::Keyed` with default parameters (no known items or generics).
    #[inline]
    pub fn new_keyed() -> Self {
        Self::Keyed(TKeyedArray::new())
    }

    /// Creates a new `ArrayType::Keyed` with the specified generic key and value types.
    #[inline]
    pub fn new_keyed_with_generics(key_type: Box<TUnion>, value_type: Box<TUnion>) -> Self {
        Self::Keyed(TKeyedArray::new_with_parameters(key_type, value_type))
    }

    /// Checks if this represents a list (`list<T>`).
    #[inline]
    pub const fn is_list(&self) -> bool {
        matches!(self, TArray::List(_))
    }

    /// Checks if this represents a keyed array (`array<Tk, Tv>`).
    #[inline]
    pub const fn is_keyed(&self) -> bool {
        matches!(self, TArray::Keyed(_))
    }

    /// Returns a reference to the `ListArrayType` data if this is a `List` variant.
    #[inline]
    pub const fn get_list(&self) -> Option<&TList> {
        if let TArray::List(data) = self { Some(data) } else { None }
    }

    /// Returns a reference to the `KeyedArrayType` data if this is a `Keyed` variant.
    #[inline]
    pub const fn get_keyed(&self) -> Option<&TKeyedArray> {
        if let TArray::Keyed(data) = self { Some(data) } else { None }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        if self.is_non_empty() {
            return false;
        }

        match &self {
            Self::Keyed(keyed_array) => {
                keyed_array.parameters.is_none()
                    && keyed_array.known_items.as_ref().is_none_or(|items| items.is_empty())
            }
            Self::List(list) => {
                list.element_type.is_never() && list.known_elements.as_ref().is_none_or(|elements| elements.is_empty())
            }
        }
    }

    #[inline]
    pub fn has_known_items(&self) -> bool {
        match &self {
            Self::Keyed(keyed_array) => keyed_array.known_items.as_ref().is_some_and(|items| !items.is_empty()),
            Self::List(list) => list.known_elements.as_ref().is_some_and(|items| !items.is_empty()),
        }
    }

    pub fn is_sealed(&self) -> bool {
        if !self.has_known_items() {
            return false;
        }

        match &self {
            Self::Keyed(keyed_array) => keyed_array.parameters.is_none(),
            Self::List(list) => list.element_type.is_never(),
        }
    }

    /// Checks if the array is known to be non-empty.
    #[inline]
    pub const fn is_non_empty(&self) -> bool {
        match &self {
            Self::Keyed(keyed_array) => keyed_array.non_empty,
            Self::List(list) => list.non_empty,
        }
    }

    /// Returns the minimum size of the array based on known items or elements.
    pub fn get_minimum_size(&self) -> usize {
        let mut size = 0;

        match &self {
            Self::Keyed(keyed_array) => {
                if let Some(known_items) = keyed_array.known_items.as_ref() {
                    for (optional, _) in known_items.values() {
                        if !optional {
                            size += 1;
                        }
                    }
                } else if keyed_array.non_empty {
                    size = 1;
                }
            }
            Self::List(list) => {
                if let Some(count) = list.known_count {
                    size = count;
                } else if let Some(known_elements) = list.known_elements.as_ref() {
                    for (optional, _) in known_elements.values() {
                        if !optional {
                            size += 1;
                        }
                    }
                } else if list.non_empty {
                    size = 1;
                }
            }
        }

        size
    }

    /// Returns the key type of the array, if applicable.
    pub fn get_key_type(&self) -> Option<TUnion> {
        match self {
            Self::Keyed(keyed_array) => {
                if let Some(parameters) = keyed_array.parameters.as_ref() {
                    return Some(parameters.0.as_ref().clone());
                }

                None
            }
            Self::List(_) => Some(get_int()),
        }
    }

    /// Returns the value type of the array, if available.
    pub fn get_value_type(&self) -> Option<TUnion> {
        match self {
            Self::Keyed(keyed_array) => {
                if let Some(parameters) = keyed_array.parameters.as_ref() {
                    return Some(parameters.1.as_ref().clone());
                }

                None
            }
            Self::List(list) => Some(list.element_type.as_ref().clone()),
        }
    }

    /// Checks if the array is truthy (non-empty or contains known definite elements).
    #[inline]
    pub fn is_truthy(&self) -> bool {
        match &self {
            Self::Keyed(keyed_array) => {
                if keyed_array.non_empty {
                    return true;
                }

                if let Some(known_items) = keyed_array.get_known_items() {
                    for (optional, _) in known_items.values() {
                        if !optional {
                            return true;
                        }
                    }
                }

                false
            }
            Self::List(list) => {
                if list.non_empty {
                    return true;
                }

                if let Some(known_elements) = list.get_known_elements() {
                    for (optional, _) in known_elements.values() {
                        if !optional {
                            return true;
                        }
                    }
                }

                false
            }
        }
    }

    /// Checks if the array is falsy (empty or contains no known elements).
    #[inline]
    pub fn is_falsy(&self) -> bool {
        match &self {
            Self::Keyed(keyed_array) => {
                if keyed_array.known_items.is_some() {
                    return false;
                }

                if keyed_array
                    .parameters
                    .as_ref()
                    .is_some_and(|parameters| !parameters.0.is_never() && !parameters.1.is_never())
                {
                    return false;
                }

                !keyed_array.non_empty
            }
            Self::List(list) => {
                if list.known_elements.is_none() && list.element_type.is_never() && !list.non_empty {
                    return true;
                }

                false
            }
        }
    }

    /// Removes placeholder types from the array type.
    #[inline]
    pub fn remove_placeholders(&mut self) {
        match self {
            Self::Keyed(keyed_array) => {
                if let Some(parameters) = keyed_array.parameters.as_mut() {
                    if let TAtomic::Placeholder = parameters.0.get_single() {
                        parameters.0 = Box::new(get_arraykey());
                    }

                    if let TAtomic::Placeholder = parameters.1.get_single() {
                        parameters.1 = Box::new(get_mixed());
                    }
                }
            }
            Self::List(list) => {
                if let TAtomic::Placeholder = list.element_type.get_single() {
                    list.element_type = Box::new(get_mixed());
                }
            }
        }
    }
}

impl TType for TArray {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        match self {
            TArray::Keyed(keyed_array) => keyed_array.get_child_nodes(),
            TArray::List(list) => list.get_child_nodes(),
        }
    }

    fn needs_population(&self) -> bool {
        match self {
            TArray::Keyed(keyed_array) => keyed_array.needs_population(),
            TArray::List(list) => list.needs_population(),
        }
    }

    fn is_expandable(&self) -> bool {
        match self {
            TArray::Keyed(keyed_array) => keyed_array.is_expandable(),
            TArray::List(list) => list.is_expandable(),
        }
    }

    fn get_id(&self) -> Atom {
        match self {
            TArray::List(list_data) => list_data.get_id(),
            TArray::Keyed(keyed_data) => keyed_data.get_id(),
        }
    }
}
