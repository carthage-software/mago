use std::sync::Arc;

use mago_word::Word;
use mago_word::concat_word;
use mago_word::join_words;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::union::TUnion;

/// An intersection whose base type must be resolved before the intersection
/// can be attached to its concrete atomic results.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TDerivedIntersection {
    base_type: Arc<TUnion>,
    intersection_types: Vec<TAtomic>,
}

impl TDerivedIntersection {
    #[must_use]
    pub fn new(base_type: TUnion) -> Self {
        Self { base_type: Arc::new(base_type), intersection_types: Vec::new() }
    }

    #[inline]
    #[must_use]
    pub fn get_base_type(&self) -> &TUnion {
        &self.base_type
    }

    #[inline]
    pub fn get_base_type_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.base_type)
    }
}

impl TType for TDerivedIntersection {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        let mut children = Vec::with_capacity(1 + self.intersection_types.len());
        children.push(TypeRef::Union(&self.base_type));
        children.extend(self.intersection_types.iter().map(TypeRef::Atomic));
        children
    }

    fn can_be_intersected(&self) -> bool {
        true
    }

    fn get_intersection_types(&self) -> Option<&[TAtomic]> {
        Some(&self.intersection_types)
    }

    fn get_intersection_types_mut(&mut self) -> Option<&mut Vec<TAtomic>> {
        Some(&mut self.intersection_types)
    }

    fn has_intersection_types(&self) -> bool {
        !self.intersection_types.is_empty()
    }

    fn add_intersection_type(&mut self, intersection_type: TAtomic) -> bool {
        self.intersection_types.push(intersection_type);
        true
    }

    fn needs_population(&self) -> bool {
        self.base_type.needs_population() || self.intersection_types.iter().any(TType::needs_population)
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn is_complex(&self) -> bool {
        true
    }

    fn get_id(&self) -> Word {
        let mut parts = Vec::with_capacity(1 + self.intersection_types.len());
        parts.push(concat_word!("(", self.base_type.get_id(), ")"));
        parts.extend(self.intersection_types.iter().map(TType::get_id));
        join_words(&parts, b"&")
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Word {
        self.get_id()
    }
}
