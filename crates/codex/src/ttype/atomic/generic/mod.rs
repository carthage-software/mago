use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::concat_atom;

use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::union::TUnion;

/// Represents a generic type parameter (`@template T of Bound`), potentially with intersection constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TGenericParameter {
    /// The name of the template parameter (e.g., `T` in `@template T`).
    pub parameter_name: Atom,
    /// The upper bound or constraint (`Bound` in `T of Bound`), represented as a type union.
    pub constraint: Box<TUnion>,
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
    pub fn new(parameter_name: Atom, constraint: Box<TUnion>, defining_entity: GenericParent) -> Self {
        Self { parameter_name, constraint, defining_entity, intersection_types: None }
    }

    /// Returns the name identifier of the template parameter.
    #[inline]
    pub const fn get_parameter_name(&self) -> Atom {
        self.parameter_name
    }

    /// Returns a reference to the main bound (`as`) type (`TUnion`).
    #[inline]
    pub fn get_constraint(&self) -> &TUnion {
        &self.constraint
    }

    /// Returns the defining entity (scope) of the template parameter.
    #[inline]
    pub const fn get_defining_entity(&self) -> GenericParent {
        self.defining_entity
    }

    pub fn is_constrained_as_numeric(&self) -> bool {
        self.constraint.is_numeric()
    }

    pub fn is_constrained_as_mixed(&self) -> bool {
        self.constraint.is_mixed()
    }

    pub fn is_constrained_as_objecty(&self) -> bool {
        self.constraint.is_objecty()
    }

    pub fn with_constraint(&self, constraint: TUnion) -> Self {
        Self {
            parameter_name: self.parameter_name,
            constraint: Box::new(constraint),
            defining_entity: self.defining_entity,
            intersection_types: self.intersection_types.clone(),
        }
    }

    pub fn without_intersection_types(&self) -> Self {
        Self {
            parameter_name: self.parameter_name,
            constraint: self.constraint.clone(),
            defining_entity: self.defining_entity,
            intersection_types: None,
        }
    }
}

impl TType for TGenericParameter {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
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

    fn get_id(&self) -> Atom {
        let base_id = concat_atom!(
            "'",
            self.parameter_name.as_str(),
            ".",
            self.defining_entity.to_string().as_str(),
            " extends ",
            self.constraint.get_id().as_str()
        );

        let Some(intersection_types) = &self.intersection_types else {
            return base_id;
        };

        let mut result = concat_atom!("(", base_id.as_str(), ")");

        for atomic in intersection_types {
            let atomic_id = atomic.get_id();
            if atomic.has_intersection_types() {
                result = concat_atom!(result.as_str(), "&(", atomic_id.as_str(), ")");
            } else {
                result = concat_atom!(result.as_str(), "&", atomic_id.as_str());
            }
        }

        result
    }
}
