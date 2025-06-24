use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;

use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::union::TUnion;

/// Specifies the kind of class-like structure a string refers to.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub enum TClassLikeStringKind {
    /// The string refers to a class name (`class-string`).
    Class,
    /// The string refers to an interface name (`interface-string`).
    Interface,
    /// The string refers to an enum name (`enum-string`).
    Enum,
    /// The string refers to a trait name (`trait-string`).
    Trait,
}

/// Represents a string that is specifically the name of a class, interface, or enum,
/// often constrained by a type (`T` in `*-string<T>`).
///
/// Examples: `class-string`, `interface-string<MyInterface>`, `enum-string<MyEnum>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub enum TClassLikeString {
    Any {
        kind: TClassLikeStringKind,
    },
    Generic {
        kind: TClassLikeStringKind,
        parameter_name: StringIdentifier,
        defining_entity: GenericParent,
        constraint: Box<TAtomic>,
    },
    Literal {
        value: StringIdentifier,
    },
    OfType {
        kind: TClassLikeStringKind,
        constraint: Box<TAtomic>,
    },
}

impl TClassLikeString {
    /// Creates a new `class-string` instance with a specific kind.
    #[inline]
    pub fn any(kind: TClassLikeStringKind) -> Self {
        Self::Any { kind }
    }

    /// Creates a new `class-string<T>` instance with a specific kind.
    #[inline]
    pub fn of_type(kind: TClassLikeStringKind, constraint: TAtomic) -> Self {
        Self::OfType { kind, constraint: Box::new(constraint) }
    }

    /// Creates a new `class-string<T>` instance with a generic parameter.
    #[inline]
    pub fn generic(
        kind: TClassLikeStringKind,
        parameter_name: StringIdentifier,
        defining_entity: GenericParent,
        constraint: TAtomic,
    ) -> Self {
        Self::Generic { kind, parameter_name, defining_entity, constraint: Box::new(constraint) }
    }

    /// Creates a new `class-string` instance with a literal value.
    #[inline]
    pub fn literal(value: StringIdentifier) -> Self {
        Self::Literal { value }
    }

    /// Creates a new `class-string` instance.
    #[inline]
    pub fn class_string() -> Self {
        Self::Any { kind: TClassLikeStringKind::Class }
    }

    /// Creates a new `class-string<T>` instance.
    #[inline]
    pub fn class_string_of_type(constraint: TAtomic) -> Self {
        Self::OfType { kind: TClassLikeStringKind::Class, constraint: Box::new(constraint) }
    }

    /// Creates a new `interface-string` instance.
    #[inline]
    pub fn interface_string() -> Self {
        Self::Any { kind: TClassLikeStringKind::Interface }
    }

    /// Creates a new `interface-string<T>` instance.
    #[inline]
    pub fn interface_string_of_type(constraint: TAtomic) -> Self {
        Self::OfType { kind: TClassLikeStringKind::Interface, constraint: Box::new(constraint) }
    }

    /// Creates a new `enum-string` instance.
    #[inline]
    pub fn enum_string() -> Self {
        Self::Any { kind: TClassLikeStringKind::Enum }
    }

    /// Creates a new `enum-string<T>` instance.
    #[inline]
    pub fn enum_string_of_type(constraint: TAtomic) -> Self {
        Self::OfType { kind: TClassLikeStringKind::Enum, constraint: Box::new(constraint) }
    }

    /// Creates a new `trait-string` instance.
    #[inline]
    pub fn trait_string() -> Self {
        Self::Any { kind: TClassLikeStringKind::Trait }
    }

    /// Creates a new `trait-string<T>` instance.
    #[inline]
    pub fn trait_string_of_type(constraint: TAtomic) -> Self {
        Self::OfType { kind: TClassLikeStringKind::Trait, constraint: Box::new(constraint) }
    }

    /// Checks if this represents a general class-like string (`Any` variant).
    #[inline]
    pub const fn is_any(&self) -> bool {
        matches!(self, Self::Any { .. })
    }

    /// Checks if this represents a class-like string derived from a generic parameter (`Generic` variant).
    #[inline]
    pub const fn is_generic(&self) -> bool {
        matches!(self, Self::Generic { .. })
    }

    /// Checks if this represents a literal class-like string with a known name (`Literal` variant).
    #[inline]
    pub const fn is_literal(&self) -> bool {
        matches!(self, Self::Literal { .. })
    }

    /// Checks if this represents a class-like string with a specific constraint type `<T>` (`OfType` variant).
    #[inline]
    pub const fn is_of_type(&self) -> bool {
        matches!(self, Self::OfType { .. })
    }

    /// Checks if the *kind* is explicitly Class (for `Any`, `Generic`, `OfType`). Returns `false` for `Literal`.
    #[inline]
    pub const fn is_class_kind(&self) -> bool {
        matches!(
            self,
            Self::Any { kind: TClassLikeStringKind::Class }
                | Self::Generic { kind: TClassLikeStringKind::Class, .. }
                | Self::OfType { kind: TClassLikeStringKind::Class, .. }
        )
    }

    /// Checks if the *kind* is explicitly Interface (for `Any`, `Generic`, `OfType`). Returns `false` for `Literal`.
    #[inline]
    pub const fn is_interface_kind(&self) -> bool {
        matches!(
            self,
            Self::Any { kind: TClassLikeStringKind::Interface }
                | Self::Generic { kind: TClassLikeStringKind::Interface, .. }
                | Self::OfType { kind: TClassLikeStringKind::Interface, .. }
        )
    }

    /// Checks if the *kind* is explicitly Enum (for `Any`, `Generic`, `OfType`). Returns `false` for `Literal`.
    #[inline]
    pub const fn is_enum_kind(&self) -> bool {
        matches!(
            self,
            Self::Any { kind: TClassLikeStringKind::Enum }
                | Self::Generic { kind: TClassLikeStringKind::Enum, .. }
                | Self::OfType { kind: TClassLikeStringKind::Enum, .. }
        )
    }

    /// Checks if this type has an explicit constraint `<T>` (`Generic` or `OfType` variants).
    #[inline]
    pub const fn has_constraint(&self) -> bool {
        matches!(self, Self::Generic { .. } | Self::OfType { .. })
    }

    /// Returns the base kind (class, interface, enum) if explicitly stored (`Any`, `Generic`, `OfType`).
    /// Returns `None` for the `Literal` variant, as the kind must be looked up externally.
    #[inline]
    pub const fn kind(&self) -> Option<TClassLikeStringKind> {
        match self {
            Self::Any { kind } => Some(*kind),
            Self::Generic { kind, .. } => Some(*kind),
            Self::Literal { .. } => None,
            Self::OfType { kind, .. } => Some(*kind),
        }
    }

    /// Returns the literal string value (class/interface/enum name) if this is a `Literal` variant.
    #[inline]
    pub fn literal_value(&self) -> Option<StringIdentifier> {
        match self {
            Self::Literal { value } => Some(*value),
            _ => None,
        }
    }

    /// Returns the constraint type `<T>` if this is a `Generic` or `OfType` variant.
    #[inline]
    pub fn constraint(&self) -> Option<&TAtomic> {
        match self {
            Self::Generic { constraint, .. } => Some(constraint),
            Self::OfType { constraint, .. } => Some(constraint),
            _ => None,
        }
    }

    /// Returns the generic parameter name if this is a `Generic` variant.
    #[inline]
    pub fn generic_parameter_name(&self) -> Option<StringIdentifier> {
        match self {
            Self::Generic { parameter_name, .. } => Some(*parameter_name),
            _ => None,
        }
    }

    /// Returns the defining entity (scope) if this is a `Generic` variant.
    #[inline]
    pub fn generic_defining_entity(&self) -> Option<&GenericParent> {
        match self {
            Self::Generic { defining_entity, .. } => Some(defining_entity),
            _ => None,
        }
    }

    /// Returns the atomic type representation of the object type this string refers to.
    #[inline]
    pub fn get_object_type(&self) -> TAtomic {
        match self {
            TClassLikeString::Any { .. } => TAtomic::Object(TObject::Any),
            TClassLikeString::Generic { parameter_name, defining_entity, constraint, .. } => {
                TAtomic::GenericParameter(TGenericParameter::new(
                    *parameter_name,
                    Box::new(TUnion::new(vec![constraint.as_ref().clone()])),
                    *defining_entity,
                ))
            }
            TClassLikeString::Literal { value } => TAtomic::Object(TObject::Named(TNamedObject::new(*value))),
            TClassLikeString::OfType { constraint, .. } => constraint.as_ref().clone(),
        }
    }
}

impl TType for TClassLikeString {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        let mut children = vec![];

        if let Some(constraint) = self.constraint() {
            children.push(TypeRef::Atomic(constraint));
        }

        children
    }

    fn get_id(&self, interner: Option<&ThreadedInterner>) -> String {
        match self {
            TClassLikeString::Any { kind } => kind.to_string(),
            TClassLikeString::Generic { kind, parameter_name, defining_entity, constraint, .. } => {
                format!(
                    "{kind}<'{}.{} extends {}>",
                    if let Some(interner) = interner {
                        interner.lookup(parameter_name).to_string()
                    } else {
                        parameter_name.to_string()
                    },
                    defining_entity.to_string(interner),
                    constraint.get_id(interner)
                )
            }
            TClassLikeString::Literal { value } => match interner {
                Some(interner) => format!("class-string<{}>", interner.lookup(value)),
                None => format!("class-string<{value}>"),
            },
            TClassLikeString::OfType { kind, constraint } => {
                format!("{kind}<{}>", constraint.get_id(interner))
            }
        }
    }
}

impl std::fmt::Display for TClassLikeStringKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TClassLikeStringKind::Class => write!(f, "class-string"),
            TClassLikeStringKind::Interface => write!(f, "interface-string"),
            TClassLikeStringKind::Enum => write!(f, "enum-string"),
            TClassLikeStringKind::Trait => write!(f, "trait-string"),
        }
    }
}
