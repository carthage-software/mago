use std::sync::Arc;

use mago_word::Word;
use mago_word::concat_word;

use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::union::TUnion;

/// Represents a generic type parameter (`@template T of Bound`), potentially with intersection constraints.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TGenericParameter {
    /// The name of the template parameter (e.g., `T` in `@template T`).
    pub parameter_name: Word,
    /// The upper bound or constraint (`Bound` in `T of Bound`), represented as a type union.
    pub constraint: Arc<TUnion>,
    /// The scope (class-like or function-like) where this template parameter was defined.
    pub defining_entity: GenericParent,
    /// Additional types intersected with this generic parameter (e.g., `&Other` in `T&Other`).
    /// Contains boxed atomic types (`TAtomic`) because intersections can involve various types.
    pub intersection_types: Option<Vec<TAtomic>>,
}

impl TGenericParameter {
    /// Creates new metadata for a generic parameter with its main bound.
    /// Initializes with no intersection types.
    ///
    /// # Arguments
    ///
    /// * `parameter_name`: The name of the template parameter (e.g., `T`).
    /// * `constraint`: The primary bound (`TUnion`), boxed (e.g., `of SomeInterface`).
    /// * `defining_entity`: The scope (`GenericParent`) where it was defined.
    #[inline]
    #[must_use]
    pub fn new(parameter_name: Word, constraint: Arc<TUnion>, defining_entity: GenericParent) -> Self {
        Self { parameter_name, constraint, defining_entity, intersection_types: None }
    }

    /// Returns the name identifier of the template parameter.
    #[inline]
    #[must_use]
    pub const fn get_parameter_name(&self) -> Word {
        self.parameter_name
    }

    /// Returns a reference to the main bound (`as`) type (`TUnion`).
    #[inline]
    #[must_use]
    pub fn get_constraint(&self) -> &TUnion {
        &self.constraint
    }

    /// Returns the defining entity (scope) of the template parameter.
    #[inline]
    #[must_use]
    pub const fn get_defining_entity(&self) -> GenericParent {
        self.defining_entity
    }

    #[must_use]
    pub fn is_constrained_as_numeric(&self) -> bool {
        self.constraint.is_numeric()
    }

    #[must_use]
    pub fn is_constrained_as_mixed(&self) -> bool {
        self.constraint.is_mixed()
    }

    #[must_use]
    pub fn is_constrained_as_vanilla_mixed(&self) -> bool {
        self.constraint.is_mixed()
    }

    #[must_use]
    pub fn is_constrained_as_objecty(&self) -> bool {
        self.constraint.is_objecty()
    }

    #[must_use]
    pub fn with_constraint(&self, constraint: TUnion) -> Self {
        Self {
            parameter_name: self.parameter_name,
            constraint: Arc::new(constraint),
            defining_entity: self.defining_entity,
            intersection_types: self.intersection_types.clone(),
        }
    }

    #[must_use]
    pub fn without_intersection_types(&self) -> Self {
        Self {
            parameter_name: self.parameter_name,
            constraint: Arc::clone(&self.constraint),
            defining_entity: self.defining_entity,
            intersection_types: None,
        }
    }
}

impl TType for TGenericParameter {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        let children = vec![TypeRef::Union(&self.constraint)];

        if let Some(intersection_types) = &self.intersection_types {
            children.into_iter().chain(intersection_types.iter().map(TypeRef::Atomic)).collect()
        } else {
            children
        }
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
        true
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn is_complex(&self) -> bool {
        self.constraint.is_complex()
    }

    fn get_id(&self) -> Word {
        let base_id = concat_word!(
            b"'",
            self.parameter_name.as_bytes(),
            b".",
            self.defining_entity.id_word().as_bytes(),
            b" extends ",
            self.constraint.get_id().as_bytes()
        );

        let Some(intersection_types) = &self.intersection_types else {
            return base_id;
        };

        let mut result = concat_word!(b"(", base_id.as_bytes(), b")");

        for atomic in intersection_types {
            let atomic_id = atomic.get_id();
            if atomic.has_intersection_types() {
                result = concat_word!(result.as_bytes(), b"&(", atomic_id.as_bytes(), b")");
            } else {
                result = concat_word!(result.as_bytes(), b"&", atomic_id.as_bytes());
            }
        }

        result
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Word {
        self.get_id()
    }
}
