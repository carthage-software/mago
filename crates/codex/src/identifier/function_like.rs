use mago_database::file::File;

use mago_span::Span;
use mago_word::Word;

use crate::identifier::method::MethodIdentifier;

/// Identifies a specific function-like construct within the codebase.
///
/// This distinguishes between globally/namespaced defined functions, methods within
/// class-like structures, and closures identified by their synthetic name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FunctionLikeIdentifier {
    /// A globally or namespaced defined function.
    /// * `Word` - The fully qualified name (FQN) of the function.
    Function(Word),
    /// A method within a class, interface, trait, or enum.
    /// * `Word` - The fully qualified class name (FQCN) of the containing structure.
    /// * `Word` - The name of the method.
    Method(Word, Word),
    /// A closure (anonymous function `function() {}` or arrow function `fn() => expr`).
    ///
    /// * `Word` - The synthetic display name produced by
    ///   [`crate::build_synthetic_name`], for example
    ///   `{closure:src/foo.php:12:5}`. The same word doubles as the
    ///   `function_likes` HashMap key and as the user-facing identifier in
    ///   issue messages, so the name is stable across machines and rebuilds.
    Closure(Word),
}

impl FunctionLikeIdentifier {
    #[inline]
    #[must_use]
    pub fn for_closure(file: &File, span: Span) -> Self {
        Self::Closure(crate::build_synthetic_name("closure", file, span))
    }

    /// Checks if this identifier represents a `Function`.
    #[inline]
    #[must_use]
    pub const fn is_function(&self) -> bool {
        matches!(self, FunctionLikeIdentifier::Function(_))
    }

    /// Checks if this identifier represents a `Method`.
    #[inline]
    #[must_use]
    pub const fn is_method(&self) -> bool {
        matches!(self, FunctionLikeIdentifier::Method(_, _))
    }

    /// Checks if this identifier represents a `Closure`.
    #[inline]
    #[must_use]
    pub const fn is_closure(&self) -> bool {
        matches!(self, FunctionLikeIdentifier::Closure(_))
    }

    /// If this identifier represents a method, returns it as a `MethodIdentifier`.
    /// Otherwise, returns `None`.
    #[inline]
    #[must_use]
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
    #[must_use]
    pub const fn title_kind_str(&self) -> &'static str {
        match self {
            FunctionLikeIdentifier::Function(_) => "Function",
            FunctionLikeIdentifier::Method(_, _) => "Method",
            FunctionLikeIdentifier::Closure(_) => "Closure",
        }
    }

    /// Returns a string representation of the kind of function-like construct.
    #[inline]
    #[must_use]
    pub const fn kind_str(&self) -> &'static str {
        match self {
            FunctionLikeIdentifier::Function(_) => "function",
            FunctionLikeIdentifier::Method(_, _) => "method",
            FunctionLikeIdentifier::Closure(_) => "closure",
        }
    }

    /// Converts the identifier to a human-readable string representation.
    ///
    /// Functions and methods render as `name` and `Class::method`. Closures
    /// render as their synthetic name verbatim, e.g. `{closure:src/foo.php:12:5}`.
    #[inline]
    #[must_use]
    pub fn as_string(&self) -> String {
        match self {
            FunctionLikeIdentifier::Function(fn_name) => fn_name.to_string(),
            FunctionLikeIdentifier::Method(fq_classlike_name, method_name) => {
                format!("{fq_classlike_name}::{method_name}")
            }
            FunctionLikeIdentifier::Closure(name) => name.to_string(),
        }
    }

    /// Creates a stable string representation suitable for use as a key or unique ID.
    #[inline]
    #[must_use]
    pub fn to_hash(&self) -> String {
        self.as_string()
    }
}

impl From<MethodIdentifier> for FunctionLikeIdentifier {
    #[inline]
    fn from(value: MethodIdentifier) -> Self {
        FunctionLikeIdentifier::Method(value.get_class_name(), value.get_method_name())
    }
}
