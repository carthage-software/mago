use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::union::TUnion;

/// Represents an instance of a specific named class, interface, or trait.
///
/// This structure holds the name, any concrete type parameters for generics,
/// flags (`$this`, internal state), and potential intersection types (`&OtherType`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TNamedObject {
    /// The fully qualified class name (FQCN) of the primary type (e.g., `A` in `A&B<T>&S`).
    pub name: StringIdentifier,
    /// Concrete types provided for generic type parameters, if any.
    pub type_parameters: Option<Vec<TUnion>>,
    /// `true` if this specifically represents the `$this` variable within its own class context.
    pub is_this: bool,
    /// Represents additional types in an intersection type (`&B&S` part of `A&B&S`).
    /// Contains other *atomic* types (boxed due to potential recursion).
    pub intersection_types: Option<Vec<TAtomic>>,
    /// Internal analysis flag: `true` if the type parameters have been remapped.
    pub remapped_parameters: bool,
}

impl TNamedObject {
    /// Creates metadata for a named object type with default flags and no generics/intersections.
    #[inline]
    pub fn new(name: StringIdentifier) -> Self {
        Self { name, type_parameters: None, is_this: false, remapped_parameters: false, intersection_types: None }
    }

    /// Creates metadata for a named object type with specified type parameters.
    #[inline]
    pub fn new_with_type_parameters(name: StringIdentifier, type_parameters: Option<Vec<TUnion>>) -> Self {
        Self { name, type_parameters, is_this: false, remapped_parameters: false, intersection_types: None }
    }

    /// Creates metadata representing the `$this` variable for a specific class.
    #[inline]
    pub fn new_this(name: StringIdentifier) -> Self {
        Self { name, type_parameters: None, is_this: true, remapped_parameters: false, intersection_types: None }
    }

    /// Returns the `StringIdentifier` for the primary class/interface name.
    #[inline]
    pub const fn get_name(&self) -> StringIdentifier {
        self.name
    }

    /// Returns the `StringIdentifier` for the primary class/interface name.
    #[inline]
    pub const fn get_name_ref(&self) -> &StringIdentifier {
        &self.name
    }

    /// Checks if this object has concrete generic type parameters.
    #[inline]
    pub fn has_type_parameters(&self) -> bool {
        self.type_parameters.as_ref().is_some_and(|v| !v.is_empty())
    }

    /// Returns a slice of the concrete generic type parameters, if specified.
    #[inline]
    pub fn get_type_parameters(&self) -> Option<&[TUnion]> {
        self.type_parameters.as_deref()
    }

    /// Returns a mutable slice of the concrete generic type parameters, if specified.
    #[inline]
    pub fn get_type_parameters_mut(&mut self) -> Option<&mut [TUnion]> {
        self.type_parameters.as_deref_mut()
    }

    /// Checks if this represents the `$this` variable.
    #[inline]
    pub const fn is_this(&self) -> bool {
        self.is_this
    }

    /// Checks if this is part of an intersection type (has extra types).
    #[inline]
    pub fn is_intersection(&self) -> bool {
        self.intersection_types.as_ref().is_some_and(|v| !v.is_empty())
    }

    /// Returns a new instance with the type parameters set.
    #[inline]
    pub fn with_type_parameters(mut self, type_parameters: Option<Vec<TUnion>>) -> Self {
        self.type_parameters = type_parameters;
        self
    }

    /// Returns a new instance with the `$this` flag set.
    #[inline]
    pub fn with_is_this(mut self, is_this: bool) -> Self {
        self.is_this = is_this;
        self
    }
}

impl TType for TNamedObject {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        let mut children = vec![];

        if let Some(type_parameters) = &self.type_parameters {
            for parameter in type_parameters {
                children.push(TypeRef::Union(parameter));
            }
        }

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

    fn get_id(&self, interner: Option<&mago_interner::ThreadedInterner>) -> String {
        let mut id = match interner {
            Some(interner) => interner.lookup(&self.name).to_string(),
            None => self.name.to_string(),
        };

        if let Some(parameters) = self.get_type_parameters() {
            id += "<";
            for (i, atomic) in parameters.iter().enumerate() {
                if i > 0 {
                    id += ", ";
                }

                id += &atomic.get_id(interner);
            }

            id += ">";
        }

        if let Some(intersection_types) = self.get_intersection_types() {
            id += "&";
            for (i, atomic) in intersection_types.iter().enumerate() {
                if i > 0 {
                    id += "&";
                }

                id += &atomic.get_id(interner);
            }
        }

        if self.is_this {
            id += "&static";
        }

        id
    }
}
