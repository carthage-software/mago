use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::concat_atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::get_mixed;
use crate::ttype::union::TUnion;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TIterable {
    /// The key type of the iterable (e.g., `K` in `iterable<K, V>`).
    pub key_type: Box<TUnion>,

    /// The value type of the iterable (e.g., `V` in `iterable<K, V>`).
    pub value_type: Box<TUnion>,

    /// Additional types intersected with this iterable (e.g., `&Other` in `iterable<K, V>&Other`).
    /// Contains boxed atomic types (`TAtomic`) because intersections can involve various types.
    pub intersection_types: Option<Vec<TAtomic>>,
}

impl TIterable {
    /// Creates a new iterable type with its key and value types.
    ///
    /// # Arguments
    ///
    /// * `key_type`: The key type of the iterable (e.g., `K`).
    /// * `value_type`: The value type of the iterable (e.g., `V`).
    #[inline]
    pub fn new(key_type: Box<TUnion>, value_type: Box<TUnion>) -> Self {
        Self { key_type, value_type, intersection_types: None }
    }

    /// Creates a new iterable type with the given value type,
    /// and a default key type of `Mixed`.
    pub fn of_value(value_type: Box<TUnion>) -> Self {
        Self::new(Box::new(get_mixed()), value_type)
    }

    /// Creates a new iterable with both key and value types set to `Mixed`.
    ///
    /// This is useful for cases where the specific types are not known or are generic.
    pub fn mixed() -> Self {
        Self::new(Box::new(get_mixed()), Box::new(get_mixed()))
    }

    /// Returns the key type of the iterable.
    #[inline]
    pub fn get_key_type(&self) -> &TUnion {
        &self.key_type
    }

    /// Returns a mutable reference to the key type of the iterable.
    #[inline]
    pub fn get_key_type_mut(&mut self) -> &mut TUnion {
        &mut self.key_type
    }

    /// Returns the value type of the iterable.
    #[inline]
    pub fn get_value_type(&self) -> &TUnion {
        &self.value_type
    }

    /// Returns a mutable reference to the value type of the iterable.
    #[inline]
    pub fn get_value_type_mut(&mut self) -> &mut TUnion {
        &mut self.value_type
    }
}

impl TType for TIterable {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        let mut children = vec![TypeRef::Union(&self.key_type), TypeRef::Union(&self.value_type)];

        if let Some(intersection_types) = &self.intersection_types {
            for atomic in intersection_types {
                children.push(TypeRef::Atomic(atomic));
            }
        }

        children
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
        self.key_type.needs_population() || self.value_type.needs_population()
    }

    fn is_expandable(&self) -> bool {
        self.key_type.is_expandable() || self.value_type.is_expandable()
    }

    fn get_id(&self) -> Atom {
        let base_id = concat_atom!("iterable<", self.key_type.get_id(), ", ", self.value_type.get_id(), ">");

        let Some(intersection_types) = self.intersection_types.as_deref() else {
            return base_id;
        };

        let mut result = concat_atom!("(", base_id, ")");
        for atomic in intersection_types {
            result = concat_atom!(result, "&", atomic.get_id());
        }

        result
    }
}
