use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_span::Span;

/// Represents a PHP variable identifier (e.g., `$foo`, `$this`).
/// Wraps a `StringIdentifier` which holds the interned name (including '$').
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct VariableIdentifier(
    /// The interned identifier for the variable name (e.g., "$foo").
    pub StringIdentifier,
);

/// Identifies the target of an expression, distinguishing simple variables from property accesses.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ExpressionIdentifier {
    /// A simple variable identifier.
    ///
    /// * `VariableIdentifier` - The identifier for the variable (e.g., `$foo`).
    Variable(VariableIdentifier),
    /// An instance property access (e.g., `$this->prop`, `$user->name`).
    ///
    /// * `VariableIdentifier` - The identifier for the object variable (e.g., `$this`, `$user`).
    /// * `Span` - The source code location covering the property name part (e.g., `prop` or `name`).
    /// * `StringIdentifier` - The name of the property being accessed (e.g., `prop`, `name`).
    InstanceProperty(VariableIdentifier, Span, StringIdentifier),
}

/// Identifies the scope where a generic template parameter (`@template`) is defined.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, Debug)]
pub enum GenericParent {
    /// The template is defined on a class, interface, trait, or enum.
    /// * `StringIdentifier` - The fully qualified name (FQCN) of the class-like structure.
    ClassLike(StringIdentifier),
    /// The template is defined on a function or method.
    /// * `(StringIdentifier, StringIdentifier)` - A tuple representing the function/method.
    ///   - `.0`: The FQCN of the class if it's a method, or the FQN of the function if global/namespaced.
    ///   - `.1`: The method name if it's a method, or `StringIdentifier::empty()` if it's a function.
    FunctionLike((StringIdentifier, StringIdentifier)),
}

impl GenericParent {
    /// Converts the `GenericParent` identifier to a stable string representation,
    /// optionally using an interner for human-readable names.
    ///
    /// Used for creating unique keys or debugging information related to generic contexts.
    /// The format distinguishes between class-like (`Namespace\ClassName`) and function-like
    /// parents (`fn-Namespace\functionName` or `fn-Namespace\ClassName::methodName`).
    ///
    /// # Arguments
    ///
    /// * `interner` - An optional reference to the `ThreadedInterner` for resolving names.
    #[inline]
    pub fn to_string(&self, interner: Option<&ThreadedInterner>) -> String {
        match self {
            GenericParent::ClassLike(id) => {
                if let Some(interner) = interner {
                    interner.lookup(id).to_string()
                } else {
                    id.to_string()
                }
            }
            GenericParent::FunctionLike(id) => {
                let part1 = id.0;
                let part2 = id.1;

                if part1.is_empty() {
                    if let Some(interner) = interner {
                        format!("{}()", interner.lookup(&part2))
                    } else {
                        format!("{part2}()")
                    }
                } else if let Some(interner) = interner {
                    format!("{}::{}()", interner.lookup(&part1), interner.lookup(&part2))
                } else {
                    format!("{part1}::{part2}()")
                }
            }
        }
    }
}
