use mago_word::Word;
use mago_word::concat_word;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::union::TUnion;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TReferenceMemberSelector {
    /// A wildcard member selector, e.g., `Foo::*`.
    Wildcard,
    /// A specific member name, e.g., `Foo::bar`.
    Identifier(Word),
    /// A member that starts with a specific prefix, e.g., `Foo::bar*`.
    StartsWith(Word),
    /// A member that ends with a specific suffix, e.g., `*::bar`.
    EndsWith(Word),
}

impl TReferenceMemberSelector {
    /// Returns true if this selector matches the given member name.
    #[inline]
    #[must_use]
    pub fn matches(&self, name: Word) -> bool {
        match self {
            Self::Wildcard => true,
            Self::Identifier(n) => *n == name,
            Self::StartsWith(prefix) => name.as_bytes().starts_with(prefix.as_bytes()),
            Self::EndsWith(suffix) => name.as_bytes().ends_with(suffix.as_bytes()),
        }
    }
}

/// Selector variants usable for global-constant references.
///
/// Unlike class members, a bare `*` already resolves to `Type::Wildcard` (an alias
/// for `mixed`), so the global-reference form only expresses prefix or suffix
/// matches. Bare identifiers in type positions remain class-like references —
/// PHPDoc does not treat them as constant references.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TGlobalReferenceSelector {
    /// A global constant whose name starts with a given prefix, e.g. `FILTER_FLAG_*`.
    StartsWith(Word),
    /// A global constant whose name ends with a given suffix, e.g. `*_SUFFIX`.
    EndsWith(Word),
}

impl TGlobalReferenceSelector {
    /// Returns true if this selector matches the given constant name.
    #[inline]
    #[must_use]
    pub fn matches(&self, name: Word) -> bool {
        match self {
            Self::StartsWith(prefix) => name.as_bytes().starts_with(prefix.as_bytes()),
            Self::EndsWith(suffix) => name.as_bytes().ends_with(suffix.as_bytes()),
        }
    }
}

/// Represents an unresolved reference to a symbol or a class-like member.
///
/// These require context (e.g., symbol tables, codebase analysis) to be resolved
/// into a concrete type (`TObject`, `TEnum`, constant type, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TReference {
    /// A reference to a symbol name (class, interface, trait, enum, ..etc).
    /// Example: `Foo`, `Bar<int>`, `T`.
    Symbol {
        /// The potentially qualified name identifier being referenced.
        name: Word,
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
        class_like_name: Word,
        /// The name of the member being referenced (constant name, case name).
        member_selector: TReferenceMemberSelector,
    },
    /// A wildcard reference over global constants, e.g. `FILTER_FLAG_*` or `*_SUFFIX`.
    /// Resolved at expansion time by iterating the codebase's global constants and
    /// collecting those whose name satisfies the selector.
    Global {
        /// The prefix/suffix selector used to match global constant names.
        selector: TGlobalReferenceSelector,
    },
}

impl TReference {
    /// Creates a simple symbol reference with no generic parameters.
    #[inline]
    #[must_use]
    pub fn new_symbol(name: Word) -> Self {
        TReference::Symbol { name, parameters: None, intersection_types: None }
    }

    /// Creates a symbol reference with generic parameters.
    #[inline]
    #[must_use]
    pub fn new_symbol_with_parameters(name: Word, parameters: Vec<TUnion>) -> Self {
        TReference::Symbol { name, parameters: Some(parameters), intersection_types: None }
    }

    /// Creates a class-like member reference.
    #[inline]
    #[must_use]
    pub fn new_member(class_like_name: Word, member_selector: TReferenceMemberSelector) -> Self {
        TReference::Member { class_like_name, member_selector }
    }

    /// Creates a global-constant wildcard reference.
    #[inline]
    #[must_use]
    pub fn new_global(selector: TGlobalReferenceSelector) -> Self {
        TReference::Global { selector }
    }

    /// Checks if this is a reference to a symbol name.
    #[inline]
    #[must_use]
    pub const fn is_symbol(&self) -> bool {
        matches!(self, TReference::Symbol { .. })
    }

    /// Checks if this is a reference to a class-like member.
    #[inline]
    #[must_use]
    pub const fn is_member(&self) -> bool {
        matches!(self, TReference::Member { .. })
    }

    /// Returns the name and parameters if this is a Symbol reference.
    #[inline]
    #[allow(clippy::type_complexity)]
    #[must_use]
    pub const fn get_symbol_data(&self) -> Option<(Word, &Option<Vec<TUnion>>, &Option<Vec<TAtomic>>)> {
        match self {
            TReference::Symbol { name, parameters, intersection_types } => {
                Some((*name, parameters, intersection_types))
            }
            _ => None,
        }
    }

    /// Returns the class-like name and member name if this is a Member reference.
    #[inline]
    #[must_use]
    pub const fn get_member_data(&self) -> Option<(Word, &TReferenceMemberSelector)> {
        match self {
            TReference::Member { class_like_name: classlike_name, member_selector } => {
                Some((*classlike_name, member_selector))
            }
            _ => None,
        }
    }
}

impl TType for TReference {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
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

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Word {
        match self {
            TReference::Symbol { name, .. } => {
                concat_word!(b"unknown-ref(", name, b")")
            }
            TReference::Member { class_like_name, member_selector } => match member_selector {
                TReferenceMemberSelector::Wildcard => {
                    concat_word!(b"unknown-ref(", class_like_name, b"::*)")
                }
                TReferenceMemberSelector::Identifier(member_name) => {
                    concat_word!(b"unknown-ref(", class_like_name, b"::", member_name, b")")
                }
                TReferenceMemberSelector::StartsWith(member_name) => {
                    concat_word!(b"unknown-ref(", class_like_name, b"::", member_name, b"*)")
                }
                TReferenceMemberSelector::EndsWith(member_name) => {
                    concat_word!(b"unknown-ref(", class_like_name, b"::*", member_name, b")")
                }
            },
            TReference::Global { selector } => match selector {
                TGlobalReferenceSelector::StartsWith(name) => {
                    concat_word!(b"unknown-global-ref(", name, b"*)")
                }
                TGlobalReferenceSelector::EndsWith(name) => {
                    concat_word!(b"unknown-global-ref(*", name, b")")
                }
            },
        }
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Word {
        self.get_id()
    }
}
