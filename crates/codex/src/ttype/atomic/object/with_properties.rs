use std::collections::BTreeMap;

use mago_word::Word;
use mago_word::word;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::union::TUnion;

/// An object type with specific known properties, as documented via `object{...}` in doc comments.
///
/// For example, `object{foo: int, bar?: string}` represents an object with a required `foo` property of type `int`
/// and an optional `bar` property of type `string`.
///
/// The `sealed` flag indicates whether the object is sealed (no additional properties will exist beyond those known).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TObjectWithProperties {
    /// Specific types known for certain keys (`Word`). The bool indicates if the element is optional.
    pub known_properties: BTreeMap<Word, (bool, TUnion)>,
    /// Whether the object is sealed (no additional properties will exist beyond those known).
    pub sealed: bool,
}

impl TObjectWithProperties {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the map of known item types by key, if any.
    #[inline]
    #[must_use]
    pub fn get_known_properties(&self) -> &BTreeMap<Word, (bool, TUnion)> {
        &self.known_properties
    }

    /// Checks if there are any known specific item types defined.
    #[inline]
    #[must_use]
    pub fn has_known_properties(&self) -> bool {
        !self.known_properties.is_empty()
    }

    /// Checks if the list contains any known indefinite elements.
    #[inline]
    #[must_use]
    pub fn has_known_indefinite_properties(&self) -> bool {
        self.known_properties.values().any(|(indefinite, _)| *indefinite)
    }
}

impl TType for TObjectWithProperties {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        let mut children = vec![];
        for (_, item_type) in self.known_properties.values() {
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

    fn get_id(&self) -> Word {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"object{");
        let mut first = true;
        for (key, (indefinite, item_type)) in &self.known_properties {
            if first {
                first = false;
            } else {
                buf.extend_from_slice(b", ");
            }

            buf.extend_from_slice(key.as_bytes());
            if *indefinite {
                buf.extend_from_slice(b"?");
            }

            buf.extend_from_slice(b": ");
            buf.extend_from_slice(item_type.get_id().as_bytes());
        }

        if !self.sealed {
            if !first {
                buf.extend_from_slice(b", ");
            }

            buf.extend_from_slice(b"...");
        }

        buf.extend_from_slice(b"}");

        word(&buf)
    }

    fn get_pretty_id_with_indent(&self, indent: usize) -> Word {
        if self.known_properties.is_empty() {
            return if self.sealed { word(b"object{}") } else { word(b"object{...}") };
        }

        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"object{\n");
        let property_indent = indent + 2;

        for (key, (indefinite, item_type)) in &self.known_properties {
            buf.resize(buf.len() + property_indent, b' ');
            buf.extend_from_slice(key.as_bytes());
            if *indefinite {
                buf.extend_from_slice(b"?");
            }
            buf.extend_from_slice(b": ");
            buf.extend_from_slice(item_type.get_pretty_id_with_indent(property_indent).as_bytes());
            buf.extend_from_slice(b",\n");
        }

        if !self.sealed {
            buf.resize(buf.len() + property_indent, b' ');
            buf.extend_from_slice(b"...,\n");
        }

        buf.resize(buf.len() + indent, b' ');
        buf.extend_from_slice(b"}");

        word(&buf)
    }
}
