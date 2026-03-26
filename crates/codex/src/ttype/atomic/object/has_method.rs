use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::concat_atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;

/// Represents an object type that has a known method from a `method_exists()` check.
///
/// This type is created when the analyzer encounters a conditional like
/// `if (method_exists($obj, 'foo'))` and narrows the type within that block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TObjectHasMethod {
    /// The name of the method that is known to exist.
    pub method: Atom,
    /// Additional intersection types (e.g., other `HasMethod` or `HasProperty` assertions).
    pub intersection_types: Option<Vec<TAtomic>>,
}

impl TObjectHasMethod {
    /// Creates a new `TObjectHasMethod` with the given method name.
    #[inline]
    #[must_use]
    pub const fn new(method: Atom) -> Self {
        Self { method, intersection_types: None }
    }

    /// Returns the method name.
    #[inline]
    #[must_use]
    pub const fn get_method(&self) -> Atom {
        self.method
    }

    /// Checks if this method name matches the given name (case-insensitive).
    #[inline]
    #[must_use]
    pub fn has_method(&self, method_name: &str) -> bool {
        self.method.eq_ignore_ascii_case(method_name)
    }
}

impl TType for TObjectHasMethod {
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
        let mut result = concat_atom!("has-method<'", self.method, "'>");

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
        let mut result = concat_atom!("has-method<'", self.method, "'>");

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
