use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::union::TUnion;

/// Metadata for a PHP array analyzed as a keyed array (map/dictionary-like).
///
/// Corresponds to `array<TKey, TValue>` or `array{'key': TVal, 1: TVal2 ...}` shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord, Default)]
pub struct TKeyedArray {
    /// Specific types known for certain keys (`ArrayKey`). The bool indicates if the element is optional.
    pub known_items: Option<BTreeMap<ArrayKey, (bool, TUnion)>>,
    /// The general key and value types (`TKey`, `TValue` in `array<TKey, TValue>`).
    /// `None` if only `known_items` are present or types are unknown/mixed.
    pub parameters: Option<(Box<TUnion>, Box<TUnion>)>,
    /// Flag indicating if the array is known to contain at least one element.
    pub non_empty: bool,
}

impl TKeyedArray {
    /// Creates new metadata for a keyed array, initially with no known items or generic parameters.
    /// Non-empty is false by default.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates new metadata for a keyed array with specified generic key and value types.
    #[inline]
    pub fn new_with_parameters(key_type: Box<TUnion>, value_type: Box<TUnion>) -> Self {
        Self { known_items: None, parameters: Some((key_type, value_type)), non_empty: false }
    }

    /// Returns a reference to the map of known item types by key, if any.
    #[inline]
    pub fn get_known_items(&self) -> Option<&BTreeMap<ArrayKey, (bool, TUnion)>> {
        self.known_items.as_ref()
    }

    /// Returns the generic key and value types (`(&TKey, &TValue)`), if specified.
    #[inline]
    pub fn get_generic_parameters(&self) -> Option<(&TUnion, &TUnion)> {
        self.parameters.as_ref().map(|(k, v)| (&**k, &**v))
    }

    /// Return a reference to the generic key type, if specified.
    #[inline]
    pub fn get_key_type(&self) -> Option<&TUnion> {
        self.parameters.as_ref().map(|(k, _)| &**k)
    }

    /// Return a reference to the generic value type, if specified.
    #[inline]
    pub fn get_value_type(&self) -> Option<&TUnion> {
        self.parameters.as_ref().map(|(_, v)| &**v)
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

    /// Checks if generic key/value parameters are defined.
    #[inline]
    pub fn has_generic_parameters(&self) -> bool {
        self.parameters.is_some()
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

impl TType for TKeyedArray {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        let mut children = vec![];
        if let Some(known_items) = self.known_items.as_ref() {
            for (_, (_, item_type)) in known_items.iter() {
                children.push(TypeRef::Union(item_type));
            }
        }

        if let Some(parameters) = self.get_generic_parameters() {
            children.push(TypeRef::Union(parameters.0));
            children.push(TypeRef::Union(parameters.1));
        }

        children
    }

    fn needs_population(&self) -> bool {
        if let Some(known_items) = &self.known_items
            && known_items.iter().any(|(_, (_, item_type))| item_type.needs_population())
        {
            return true;
        }

        if let Some(parameters) = &self.parameters {
            return parameters.0.needs_population() || parameters.1.needs_population();
        }

        false
    }

    fn is_expandable(&self) -> bool {
        if let Some(known_items) = &self.known_items
            && known_items.iter().any(|(_, (_, item_type))| item_type.is_expandable())
        {
            return true;
        }

        if let Some(parameters) = &self.parameters {
            return parameters.0.is_expandable() || parameters.1.is_expandable();
        }

        false
    }

    fn get_id(&self) -> Atom {
        if let Some(items) = &self.known_items {
            let mut string = String::new();
            string += "array{";
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

            if let Some((key_type, value_type)) = &self.parameters {
                if !first {
                    string += ", ";
                }

                string += "...";
                if !key_type.is_array_key() || !value_type.is_mixed() {
                    string += "<";
                    string += &key_type.get_id();
                    string += ", ";
                    string += &value_type.get_id();
                    string += ">";
                }
            }

            string += "}";

            atom(&string)
        } else if let Some((key_type, value_type)) = &self.parameters {
            concat_atom!(
                if self.is_non_empty() { "non-empty-array" } else { "array" },
                "<",
                key_type.get_id().as_str(),
                ", ",
                value_type.get_id().as_str(),
                ">",
            )
        } else {
            atom("array{}")
        }
    }
}
