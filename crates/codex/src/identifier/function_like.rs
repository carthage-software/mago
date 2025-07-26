use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_span::Position;

use crate::identifier::method::MethodIdentifier;

/// Identifies a specific function-like construct within the codebase.
///
/// This distinguishes between globally/namespaced defined functions, methods within
/// class-like structures, and closures identified by their source position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum FunctionLikeIdentifier {
    /// A globally or namespaced defined function.
    /// * `StringIdentifier` - The fully qualified name (FQN) of the function.
    Function(StringIdentifier),
    /// A method within a class, interface, trait, or enum.
    /// * `StringIdentifier` - The fully qualified class name (FQCN) of the containing structure.
    /// * `StringIdentifier` - The name of the method.
    Method(StringIdentifier, StringIdentifier),
    /// A closure (anonymous function `function() {}` or arrow function `fn() => expr`).
    /// * `Position` - The starting position (source file and offset) of the closure definition.
    Closure(Position),
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
        matches!(self, FunctionLikeIdentifier::Closure(_))
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
            FunctionLikeIdentifier::Closure(_) => "Closure",
        }
    }

    /// Returns a string representation of the kind of function-like construct.
    #[inline]
    pub const fn kind_str(&self) -> &'static str {
        match self {
            FunctionLikeIdentifier::Function(_) => "function",
            FunctionLikeIdentifier::Method(_, _) => "method",
            FunctionLikeIdentifier::Closure(_) => "closure",
        }
    }

    /// Converts the identifier to a human-readable string representation using the provided interner.
    ///
    /// For closures, this typically includes the filename and starting offset.
    ///
    /// # Arguments
    ///
    /// * `interner` - A reference to the `ThreadedInterner` used to resolve `StringIdentifier`s.
    #[inline]
    pub fn as_string(&self, interner: &ThreadedInterner) -> String {
        match self {
            FunctionLikeIdentifier::Function(fn_name) => interner.lookup(fn_name).to_string(),
            FunctionLikeIdentifier::Method(fq_classlike_name, method_name) => {
                format!("{}::{}", interner.lookup(fq_classlike_name), interner.lookup(method_name))
            }
            FunctionLikeIdentifier::Closure(position) => {
                format!("{}:{}", interner.lookup(&position.source.value()), position.offset)
            }
        }
    }

    /// Creates a stable string representation suitable for use as a key or unique ID,
    /// without requiring an interner lookup.
    ///
    /// This uses the internal representation of `StringIdentifier`
    /// and position information directly.
    #[inline]
    pub fn to_hash(&self) -> String {
        match self {
            FunctionLikeIdentifier::Function(fn_name) => fn_name.to_string(),
            FunctionLikeIdentifier::Method(fq_classlike_name, method_name) => {
                format!("{fq_classlike_name}::{method_name}")
            }
            FunctionLikeIdentifier::Closure(position) => {
                format!("{}::{}", position.source.value(), position.offset)
            }
        }
    }
}
