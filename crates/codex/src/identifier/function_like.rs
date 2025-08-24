use mago_database::file::FileId;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_span::Position;

use crate::identifier::method::MethodIdentifier;

/// Identifies a specific function-like construct within the codebase.
///
/// This distinguishes between globally/namespaced defined functions, methods within
/// class-like structures, and closures identified by their source position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum FunctionLikeIdentifier {
    /// A globally or namespaced defined function.
    /// * `Atom` - The fully qualified name (FQN) of the function.
    Function(Atom),
    /// A method within a class, interface, trait, or enum.
    /// * `Atom` - The fully qualified class name (FQCN) of the containing structure.
    /// * `Atom` - The name of the method.
    Method(Atom, Atom),
    /// A closure (anonymous function `function() {}` or arrow function `fn() => expr`).
    ///
    /// * `FileId` - The identifier of the file where the closure is defined.
    /// * `Position` - The starting position of the closure definition.
    Closure(FileId, Position),
}

impl FunctionLikeIdentifier {
    /// Checks if this identifier represents a `Function`.
    #[inline]
    pub const fn is_function(&self) -> bool {
        matches!(self, FunctionLikeIdentifier::Function(_))
    }

    /// Checks if this identifier represents a `Method`.
    #[inline]
    pub const fn is_method(&self) -> bool {
        matches!(self, FunctionLikeIdentifier::Method(_, _))
    }

    /// Checks if this identifier represents a `Closure`.
    #[inline]
    pub const fn is_closure(&self) -> bool {
        matches!(self, FunctionLikeIdentifier::Closure(_, _))
    }

    /// If this identifier represents a method, returns it as a `MethodIdentifier`.
    /// Otherwise, returns `None`.
    #[inline]
    pub const fn as_method_identifier(&self) -> Option<MethodIdentifier> {
        match self {
            FunctionLikeIdentifier::Method(fq_classlike_name, method_name) => {
                Some(MethodIdentifier::new(*fq_classlike_name, *method_name))
            }
            _ => None,
        }
    }

    /// Returns a string representation of the kind of function-like construct.
    #[inline]
    pub const fn title_kind_str(&self) -> &'static str {
        match self {
            FunctionLikeIdentifier::Function(_) => "Function",
            FunctionLikeIdentifier::Method(_, _) => "Method",
            FunctionLikeIdentifier::Closure(_, _) => "Closure",
        }
    }

    /// Returns a string representation of the kind of function-like construct.
    #[inline]
    pub const fn kind_str(&self) -> &'static str {
        match self {
            FunctionLikeIdentifier::Function(_) => "function",
            FunctionLikeIdentifier::Method(_, _) => "method",
            FunctionLikeIdentifier::Closure(_, _) => "closure",
        }
    }

    /// Converts the identifier to a human-readable string representation.
    ///
    /// For closures, this typically includes the filename and starting offset.
    #[inline]
    pub fn as_string(&self) -> String {
        match self {
            FunctionLikeIdentifier::Function(fn_name) => fn_name.to_string(),
            FunctionLikeIdentifier::Method(fq_classlike_name, method_name) => {
                format!("{}::{}", fq_classlike_name, method_name)
            }
            FunctionLikeIdentifier::Closure(file_id, position) => {
                format!("{}:{}", file_id, position.offset)
            }
        }
    }

    /// Creates a stable string representation suitable for use as a key or unique ID.
    #[inline]
    pub fn to_hash(&self) -> String {
        match self {
            FunctionLikeIdentifier::Function(fn_name) => fn_name.to_string(),
            FunctionLikeIdentifier::Method(fq_classlike_name, method_name) => {
                format!("{fq_classlike_name}::{method_name}")
            }
            FunctionLikeIdentifier::Closure(file_id, position) => {
                format!("{}::{}", file_id, position.offset)
            }
        }
    }
}
