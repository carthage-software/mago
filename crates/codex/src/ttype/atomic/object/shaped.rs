use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::union::TUnion;

/// Metadata for a PHP object (map/dictionary-like).
///
/// Corresponds to `object{'key': TVal, 1: TVal2 ...}` shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord, Default)]
pub struct TShapedObject {
    /// Specific types known for certain keys (`ArrayKey`). The bool indicates if the element is optional.
    pub known_items: Option<BTreeMap<ArrayKey, (bool, TUnion)>>,
    pub non_empty: bool,
}

impl TShapedObject {
    /// Creates new metadata for a keyed array, initially with no known items or generic parameters.
    /// Non-empty is false by default.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the map of known item types by key, if any.
    #[inline]
    pub fn get_known_items(&self) -> Option<&BTreeMap<ArrayKey, (bool, TUnion)>> {
        self.known_items.as_ref()
    }

    /// Checks if the array is known to be non-empty.
    #[inline]
    pub const fn is_non_empty(&self) -> bool {
        self.non_empty
    }

    /// Checks if there are any known specific item types defined.
    #[inline]
    pub fn has_known_items(&self) -> bool {
        self.known_items.as_ref().is_some_and(|elements| !elements.is_empty())
    }

    /// Checks if the list contains any known indefinite elements.
    #[inline]
    pub fn has_known_indefinite_items(&self) -> bool {
        self.known_items.as_ref().is_some_and(|elements| elements.values().any(|(indefinite, _)| *indefinite))
    }

    /// Returns a new `TKeyedArray` with the non-empty flag set to true.
    #[inline]
    pub fn to_non_empty(self) -> Self {
        Self { non_empty: true, ..self }
    }

    /// Returns a new `TKeyedArray` with the specified non-empty flag.
    #[inline]
    pub fn as_non_empty_array(&self, non_empty: bool) -> Self {
        Self { non_empty, ..self.clone() }
    }
}

impl TType for TShapedObject {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        let mut children = vec![];
        if let Some(known_items) = self.known_items.as_ref() {
            for (_, (_, item_type)) in known_items.iter() {
                children.push(TypeRef::Union(item_type));
            }
        }

        children
    }

    fn needs_population(&self) -> bool {
        if let Some(known_items) = &self.known_items
            && known_items.iter().any(|(_, (_, item_type))| item_type.needs_population())
        {
            return true;
        }

        false
    }

    fn is_expandable(&self) -> bool {
        if let Some(known_items) = &self.known_items
            && known_items.iter().any(|(_, (_, item_type))| item_type.is_expandable())
        {
            return true;
        }

        false
    }

    fn get_id(&self) -> Atom {
        if let Some(items) = &self.known_items {
            let mut string = String::new();
            string += "object{";
            let mut first = true;
            for (key, (indefinite, item_type)) in items {
                if !first {
                    string += ", ";
                } else {
                    first = false;
                }

                string += &key.to_string();
                if *indefinite {
                    string += "?";
                }

                string += ": ";
                string += &item_type.get_id();
            }

            string += "}";

            atom(&string)
        } else {
            atom("object{}")
        }
    }
}
