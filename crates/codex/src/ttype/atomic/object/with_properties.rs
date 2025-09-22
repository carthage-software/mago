use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::union::TUnion;

/// An object type with specific known properties, as documented via `object{...}` in doc comments.
///
/// For example, `object{foo: int, bar?: string}` represents an object with a required `foo` property of type `int`
/// and an optional `bar` property of type `string`.
///
/// The `sealed` flag indicates whether the object is sealed (no additional properties will exist beyond those known).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord, Default)]
pub struct TObjectWithProperties {
    /// Specific types known for certain keys (`Atom`). The bool indicates if the element is optional.
    pub known_properties: BTreeMap<Atom, (bool, TUnion)>,
    /// Whether the object is sealed (no additional properties will exist beyond those known).
    pub sealed: bool,
}

impl TObjectWithProperties {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the map of known item types by key, if any.
    #[inline]
    pub fn get_known_properties(&self) -> &BTreeMap<Atom, (bool, TUnion)> {
        &self.known_properties
    }

    /// Checks if there are any known specific item types defined.
    #[inline]
    pub fn has_known_properties(&self) -> bool {
        !self.known_properties.is_empty()
    }

    /// Checks if the list contains any known indefinite elements.
    #[inline]
    pub fn has_known_indefinite_properties(&self) -> bool {
        self.known_properties.values().any(|(indefinite, _)| *indefinite)
    }
}

impl TType for TObjectWithProperties {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        let mut children = vec![];
        for (_, (_, item_type)) in self.known_properties.iter() {
            children.push(TypeRef::Union(item_type));
        }

        children
    }

    fn needs_population(&self) -> bool {
        self.known_properties.iter().any(|(_, (_, item_type))| item_type.needs_population())
    }

    fn is_expandable(&self) -> bool {
        self.known_properties.iter().any(|(_, (_, item_type))| item_type.is_expandable())
    }

    fn is_complex(&self) -> bool {
        !self.known_properties.is_empty()
    }

    fn get_id(&self) -> Atom {
        let mut string = String::new();
        string += "object{";
        let mut first = true;
        for (key, (indefinite, item_type)) in &self.known_properties {
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

        if !self.sealed {
            if !first {
                string += ", ";
            }

            string += "...";
        }

        string += "}";

        atom(&string)
    }

    fn get_pretty_id_with_indent(&self, indent: usize) -> Atom {
        if self.known_properties.is_empty() {
            return if self.sealed { atom("object{}") } else { atom("object{...}") };
        }

        let mut string = String::new();
        string += "object{\n";
        let property_indent = indent + 2;
        let property_spaces = " ".repeat(property_indent);

        for (key, (indefinite, item_type)) in &self.known_properties {
            string += &property_spaces;
            string += key.as_ref();
            if *indefinite {
                string += "?";
            }
            string += ": ";
            string += &item_type.get_pretty_id_with_indent(property_indent);
            string += ",\n";
        }

        if !self.sealed {
            string += &property_spaces;
            string += "...,\n";
        }

        string += &" ".repeat(indent);
        string += "}";

        atom(&string)
    }
}
