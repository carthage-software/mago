use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::concat_atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::union::TUnion;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum TReferenceMemberSelector {
    /// A wildcard member selector, e.g., `Foo::*`.
    Wildcard,
    /// A specific member name, e.g., `Foo::bar`.
    Identifier(Atom),
    /// A member that starts with a specific prefix, e.g., `Foo::bar*`.
    StartsWith(Atom),
    /// A member that ends with a specific suffix, e.g., `*::bar`.
    EndsWith(Atom),
}

/// Represents an unresolved reference to a symbol or a class-like member.
/// These require context (e.g., symbol tables, codebase analysis) to be resolved
/// into a concrete type (`TObject`, `TEnum`, constant type, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum TReference {
    /// A reference to a symbol name (class, interface, trait, enum, ..etc).
    /// Example: `Foo`, `Bar<int>`, `T`.
    Symbol {
        /// The potentially qualified name identifier being referenced.
        name: Atom,
        /// Generic arguments provided at the reference site, e.g., the `<int>` in `Foo<int>`.
        /// Kept original name `type_params` as requested for fields.
        parameters: Option<Vec<TUnion>>,
        /// Represents additional types in an intersection type (`&B&S` part of `A&B&S`).
        /// Contains other *atomic* types (boxed due to potential recursion).
        intersection_types: Option<Vec<TAtomic>>,
    },
    /// A reference to a member within a class-like scope (class constant, enum case).
    /// Example: `Client::THRESHOLD`, `Status::Ok`.
    Member {
        /// The FQCN of the class-like structure containing the member.
        class_like_name: Atom,
        /// The name of the member being referenced (constant name, case name).
        member_selector: TReferenceMemberSelector,
    },
}

impl TReference {
    /// Creates a simple symbol reference with no generic parameters.
    #[inline]
    pub fn new_symbol(name: Atom) -> Self {
        TReference::Symbol { name, parameters: None, intersection_types: None }
    }

    /// Creates a symbol reference with generic parameters.
    #[inline]
    pub fn new_symbol_with_parameters(name: Atom, parameters: Vec<TUnion>) -> Self {
        TReference::Symbol { name, parameters: Some(parameters), intersection_types: None }
    }

    /// Creates a class-like member reference.
    #[inline]
    pub fn new_member(class_like_name: Atom, member_selector: TReferenceMemberSelector) -> Self {
        TReference::Member { class_like_name, member_selector }
    }

    /// Checks if this is a reference to a symbol name.
    #[inline]
    pub const fn is_symbol(&self) -> bool {
        matches!(self, TReference::Symbol { .. })
    }

    /// Checks if this is a reference to a class-like member.
    #[inline]
    pub const fn is_member(&self) -> bool {
        matches!(self, TReference::Member { .. })
    }

    /// Returns the name and parameters if this is a Symbol reference.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub const fn get_symbol_data(&self) -> Option<(&Atom, &Option<Vec<TUnion>>, &Option<Vec<TAtomic>>)> {
        match self {
            TReference::Symbol { name, parameters, intersection_types } => Some((name, parameters, intersection_types)),
            _ => None,
        }
    }

    /// Returns the class-like name and member name if this is a Member reference.
    #[inline]
    pub const fn get_member_data(&self) -> Option<(&Atom, &TReferenceMemberSelector)> {
        match self {
            TReference::Member { class_like_name: classlike_name, member_selector } => {
                Some((classlike_name, member_selector))
            }
            _ => None,
        }
    }
}

impl TType for TReference {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        let mut children = Vec::new();
        if let TReference::Symbol { parameters, intersection_types, .. } = self {
            if let Some(params) = parameters {
                for param in params {
                    children.push(TypeRef::Union(param));
                }
            }

            if let Some(intersection_types) = intersection_types {
                for atomic in intersection_types {
                    children.push(TypeRef::Atomic(atomic));
                }
            }
        }

        children
    }

    fn can_be_intersected(&self) -> bool {
        matches!(self, TReference::Symbol { .. })
    }

    fn get_intersection_types(&self) -> Option<&[TAtomic]> {
        match self {
            TReference::Symbol { intersection_types, .. } => intersection_types.as_deref(),
            _ => None,
        }
    }

    fn get_intersection_types_mut(&mut self) -> Option<&mut Vec<TAtomic>> {
        match self {
            TReference::Symbol { intersection_types, .. } => intersection_types.as_mut(),
            _ => None,
        }
    }

    fn has_intersection_types(&self) -> bool {
        match self {
            TReference::Symbol { intersection_types, .. } => intersection_types.as_ref().is_some_and(|v| !v.is_empty()),
            _ => false,
        }
    }

    fn add_intersection_type(&mut self, intersection_type: TAtomic) -> bool {
        match self {
            TReference::Symbol { intersection_types, .. } => {
                if let Some(intersection_types) = intersection_types {
                    intersection_types.push(intersection_type);
                } else {
                    *intersection_types = Some(vec![intersection_type]);
                }

                true
            }
            _ => false,
        }
    }

    fn needs_population(&self) -> bool {
        true
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn get_id(&self) -> Atom {
        match self {
            TReference::Symbol { name, .. } => {
                concat_atom!("unknown-ref(", name, ")")
            }
            TReference::Member { class_like_name, member_selector } => match member_selector {
                TReferenceMemberSelector::Wildcard => {
                    concat_atom!("unknown-ref(", class_like_name, "::*)")
                }
                TReferenceMemberSelector::Identifier(member_name) => {
                    concat_atom!("unknown-ref(", class_like_name, "::", member_name, ")")
                }
                TReferenceMemberSelector::StartsWith(member_name) => {
                    concat_atom!("unknown-ref(", class_like_name, "::", member_name, "*)")
                }
                TReferenceMemberSelector::EndsWith(member_name) => {
                    concat_atom!("unknown-ref(", class_like_name, "::*", member_name, ")")
                }
            },
        }
    }
}
