use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::get_never;
use crate::ttype::union::TUnion;

/// Metadata for a PHP array analyzed as a list (vector-like).
/// Corresponds to `list<TValue>` or `array{T0, T1, ...}` list-shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TList {
    /// The general type of elements in the list (`TValue` in `list<TValue>`).
    pub element_type: Box<TUnion>,
    /// Specific types known for certain integer indices. The bool indicates if the element is optional.
    pub known_elements: Option<BTreeMap<usize, (bool, TUnion)>>,
    /// The known exact number of elements, if determined (e.g., from `count()` or literal definition).
    pub known_count: Option<usize>,
    /// Flag indicating if the list is known to contain at least one element.
    pub non_empty: bool,
}

impl TList {
    /// Creates new metadata for a list type with the specified general element type.
    /// Initializes known elements/count as None, and non_empty as false.
    ///
    /// # Arguments
    ///
    /// * `element_type`: The general type (`TUnion`) of elements in the list, boxed.
    #[inline]
    pub fn new(element_type: Box<TUnion>) -> Self {
        Self { element_type, known_elements: None, known_count: None, non_empty: false }
    }

    /// Creates new metadata for a list type with specified known elements.
    ///
    /// If all known elements are non-optional, sets known_count accordingly.
    /// Sets non_empty to true if any known element is non-optional.
    ///
    /// # Arguments
    ///
    /// * `known_elements`: A BTreeMap mapping indices to (is_optional, TUnion) tuples.
    pub fn from_known_elements(known_elements: BTreeMap<usize, (bool, TUnion)>) -> Self {
        Self {
            element_type: Box::new(get_never()),
            known_count: if known_elements.values().all(|(optional, _)| !*optional) {
                Some(known_elements.len())
            } else {
                None
            },
            non_empty: known_elements.values().any(|(optional, _)| !*optional),
            known_elements: Some(known_elements),
        }
    }

    #[inline]
    pub fn new_non_empty(element_type: Box<TUnion>) -> Self {
        Self { element_type, known_elements: None, known_count: None, non_empty: true }
    }

    /// Returns a reference to the general element type (`TUnion`).
    #[inline]
    pub fn get_element_type(&self) -> &TUnion {
        &self.element_type
    }

    /// Returns a reference to the map of known element types by index, if any.
    #[inline]
    pub fn get_known_elements(&self) -> Option<&BTreeMap<usize, (bool, TUnion)>> {
        self.known_elements.as_ref()
    }

    /// Returns the known count of elements, if determined.
    #[inline]
    pub fn get_known_count(&self) -> Option<usize> {
        self.known_count
    }

    /// Create a non-empty clone of the list type.
    #[inline]
    pub fn clone_non_empty(&self) -> Self {
        Self {
            element_type: self.element_type.clone(),
            known_elements: self.known_elements.clone(),
            known_count: self.known_count,
            non_empty: true,
        }
    }

    /// Create a non-empty clone of the list type.
    #[inline]
    pub fn clone_non_empty_with_count(&self, count: Option<usize>) -> Self {
        Self {
            element_type: self.element_type.clone(),
            known_elements: self.known_elements.clone(),
            known_count: count,
            non_empty: true,
        }
    }

    /// Checks if the list is known to be non-empty.
    #[inline]
    pub const fn is_non_empty(&self) -> bool {
        self.non_empty
    }

    /// Checks if there are any known specific element types defined.
    #[inline]
    pub fn has_known_elements(&self) -> bool {
        self.known_elements.is_some()
    }

    /// Checks if the list contains any known optional elements.
    #[inline]
    pub fn has_known_optional_elements(&self) -> bool {
        self.known_elements.as_ref().is_some_and(|elements| elements.values().any(|(optional, _)| *optional))
    }

    /// Checks if the exact count of elements is known.
    #[inline]
    pub fn has_known_count(&self) -> bool {
        self.known_count.is_some()
    }
}

impl TType for TList {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        let mut children = vec![];
        if let Some(known_items) = self.get_known_elements() {
            for (_, (_, item_type)) in known_items.iter() {
                children.push(TypeRef::Union(item_type));
            }
        }

        children.push(TypeRef::Union(self.get_element_type()));
        children
    }

    fn needs_population(&self) -> bool {
        if let Some(elements) = &self.known_elements
            && elements.iter().any(|element| element.1.1.needs_population())
        {
            return true;
        }

        self.element_type.needs_population()
    }

    fn is_expandable(&self) -> bool {
        if let Some(elements) = &self.known_elements
            && elements.iter().any(|element| element.1.1.is_expandable())
        {
            return true;
        }

        self.element_type.is_expandable()
    }

    fn is_complex(&self) -> bool {
        if let Some(elements) = &self.known_elements
            && !elements.is_empty()
        {
            return true;
        }

        self.element_type.is_complex()
    }

    fn get_id(&self) -> Atom {
        if let Some(elements) = &self.known_elements {
            // Format as list{...} shape
            let mut string = String::new();
            string += "list{";
            let has_optional = self.has_known_optional_elements();
            let mut first = true;
            let mut include_index = false;
            for (i, (optional, element_type)) in elements {
                if !first {
                    string += ", ";
                } else {
                    first = false;
                    include_index = *i != 0;
                }

                if has_optional || include_index {
                    string += &i.to_string();
                    if *optional {
                        string += "?";
                    }

                    string += ": ";
                }

                string += &element_type.get_id();
            }

            if !self.element_type.is_never() {
                if !first {
                    string += ", ";
                }

                string += "...<";
                string += &self.element_type.get_id();
                string += ">";
            }

            string += "}";

            atom(&string)
        } else {
            concat_atom!(
                if self.is_non_empty() { "non-empty-list" } else { "list" },
                "<",
                self.element_type.get_id().as_str(),
                ">"
            )
        }
    }

    fn get_pretty_id_with_indent(&self, indent: usize) -> Atom {
        if let Some(elements) = &self.known_elements {
            if elements.is_empty() && self.element_type.is_never() {
                return atom("list{}");
            }

            let mut string = String::new();
            string += "list{\n";
            let element_indent = indent + 2;
            let element_spaces = " ".repeat(element_indent);
            let has_optional = self.has_known_optional_elements();
            let mut include_index = false;

            for (i, (optional, element_type)) in elements {
                if *i == 0 {
                    include_index = elements.len() > 1 || has_optional;
                }

                string += &element_spaces;
                if has_optional || include_index {
                    string += &i.to_string();
                    if *optional {
                        string += "?";
                    }
                    string += ": ";
                }
                string += &element_type.get_pretty_id_with_indent(element_indent);
                string += ",\n";
            }

            if !self.element_type.is_never() {
                string += &element_spaces;
                string += "...";
                if self.element_type.is_complex() {
                    string += "<\n";
                    string += &" ".repeat(element_indent + 2);
                    string += &self.element_type.get_pretty_id_with_indent(element_indent + 2);
                    string += ",\n";
                    string += &element_spaces;
                    string += ">";
                } else {
                    string += "<";
                    string += &self.element_type.get_pretty_id_with_indent(element_indent);
                    string += ">";
                }
                string += ",\n";
            }

            string += &" ".repeat(indent);
            string += "}";

            atom(&string)
        } else if self.element_type.is_complex() {
            let mut string = String::new();
            string += if self.is_non_empty() { "non-empty-list" } else { "list" };
            string += "<\n";
            string += &" ".repeat(indent + 2);
            string += &self.element_type.get_pretty_id_with_indent(indent + 2);
            string += ",\n";
            string += &" ".repeat(indent);
            string += ">";
            atom(&string)
        } else {
            concat_atom!(
                if self.is_non_empty() { "non-empty-list" } else { "list" },
                "<",
                self.element_type.get_pretty_id_with_indent(indent).as_str(),
                ">"
            )
        }
    }
}
