use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::concat_atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;

/// Represents an object type that has a known property from a `property_exists()` check.
///
/// This type is created when the analyzer encounters a conditional like
/// `if (property_exists($obj, 'foo'))` and narrows the type within that block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TObjectHasProperty {
    /// The name of the property that is known to exist.
    pub property: Atom,
    /// Additional intersection types (e.g., other `HasMethod` or `HasProperty` assertions).
    pub intersection_types: Option<Vec<TAtomic>>,
}

impl TObjectHasProperty {
    /// Creates a new `TObjectHasProperty` with the given property name.
    #[inline]
    #[must_use]
    pub const fn new(property: Atom) -> Self {
        Self { property, intersection_types: None }
    }

    /// Returns the property name.
    #[inline]
    #[must_use]
    pub const fn get_property(&self) -> &Atom {
        &self.property
    }

    /// Checks if this property name matches the given name.
    #[inline]
    #[must_use]
    pub fn has_property(&self, property_name: &Atom) -> bool {
        &self.property == property_name
    }
}

impl TType for TObjectHasProperty {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        self.intersection_types.as_ref().map(|types| types.iter().map(TypeRef::Atomic).collect()).unwrap_or_default()
    }

    fn can_be_intersected(&self) -> bool {
        true
    }

    fn get_intersection_types(&self) -> Option<&[TAtomic]> {
        self.intersection_types.as_deref()
    }

    fn get_intersection_types_mut(&mut self) -> Option<&mut Vec<TAtomic>> {
        self.intersection_types.as_mut()
    }

    fn has_intersection_types(&self) -> bool {
        self.intersection_types.as_ref().is_some_and(|v| !v.is_empty())
    }

    fn add_intersection_type(&mut self, intersection_type: TAtomic) -> bool {
        if let Some(intersection_types) = self.intersection_types.as_mut() {
            intersection_types.push(intersection_type);
        } else {
            self.intersection_types = Some(vec![intersection_type]);
        }

        true
    }

    fn needs_population(&self) -> bool {
        false
    }

    fn is_expandable(&self) -> bool {
        false
    }

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Atom {
        let mut result = concat_atom!("has-property<'", self.property, "'>");

        if let Some(intersection_types) = self.get_intersection_types() {
            for atomic in intersection_types {
                let atomic_id = atomic.get_id();

                result = if atomic.has_intersection_types() {
                    concat_atom!(result, "&(", atomic_id, ")")
                } else {
                    concat_atom!(result, "&", atomic_id)
                };
            }
        }

        result
    }

    fn get_pretty_id_with_indent(&self, indent: usize) -> Atom {
        let mut result = concat_atom!("has-property<'", self.property, "'>");

        if let Some(intersection_types) = self.get_intersection_types() {
            for atomic in intersection_types {
                let atomic_id = atomic.get_pretty_id_with_indent(indent);

                result = if atomic.has_intersection_types() {
                    concat_atom!(result, "&(", atomic_id, ")")
                } else {
                    concat_atom!(result, "&", atomic_id)
                };
            }
        }

        result
    }
}
