use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::union::TUnion;

/// Metadata for a PHP object (map/dictionary-like).
///
/// Corresponds to `object{'key': TVal, 1: TVal2 ...}` shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord, Default)]
pub struct TObjectWithProperties {
    /// Specific types known for certain keys (`Atom`). The bool indicates if the element is optional.
    pub known_properties: Option<BTreeMap<Atom, (bool, TUnion)>>,
}

impl TObjectWithProperties {
    /// Creates new metadata for a keyed array, initially with no known items or generic parameters.
    /// Non-empty is false by default.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the map of known item types by key, if any.
    #[inline]
    pub fn get_known_properties(&self) -> Option<&BTreeMap<Atom, (bool, TUnion)>> {
        self.known_properties.as_ref()
    }

    /// Checks if there are any known specific item types defined.
    #[inline]
    pub fn has_known_properties(&self) -> bool {
        self.known_properties.as_ref().is_some_and(|elements| !elements.is_empty())
    }

    /// Checks if the list contains any known indefinite elements.
    #[inline]
    pub fn has_known_indefinite_properties(&self) -> bool {
        self.known_properties.as_ref().is_some_and(|elements| elements.values().any(|(indefinite, _)| *indefinite))
    }
}

impl TType for TObjectWithProperties {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        let mut children = vec![];
        if let Some(known_items) = self.known_properties.as_ref() {
            for (_, (_, item_type)) in known_items.iter() {
                children.push(TypeRef::Union(item_type));
            }
        }

        children
    }

    fn needs_population(&self) -> bool {
        if let Some(known_items) = &self.known_properties
            && known_items.iter().any(|(_, (_, item_type))| item_type.needs_population())
        {
            return true;
        }

        false
    }

    fn is_expandable(&self) -> bool {
        if let Some(known_items) = &self.known_properties
            && known_items.iter().any(|(_, (_, item_type))| item_type.is_expandable())
        {
            return true;
        }

        false
    }

    fn get_id(&self) -> Atom {
        if let Some(items) = &self.known_properties {
            let mut string = String::new();
            string += "object{";
            let mut first = true;
            for (key, (indefinite, item_type)) in items {
                if !first {
                    string += ", ";
                } else {
                    first = false;
                }

                string += key.as_ref();
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
