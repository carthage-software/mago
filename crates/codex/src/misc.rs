use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_span::Span;

/// Represents a PHP variable identifier (e.g., `$foo`, `$this`).
/// Wraps a `Atom` which holds the interned name (including '$').
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct VariableIdentifier(
    /// The atom for the variable name (e.g., "$foo").
    pub Atom,
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
    /// * `Atom` - The name of the property being accessed (e.g., `prop`, `name`).
    InstanceProperty(VariableIdentifier, Span, Atom),
}

/// Identifies the scope where a generic template parameter (`@template`) is defined.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, Debug)]
pub enum GenericParent {
    /// The template is defined on a class, interface, trait, or enum.
    /// * `Atom` - The fully qualified name (FQCN) of the class-like structure.
    ClassLike(Atom),
    /// The template is defined on a function or method.
    /// * `(Atom, Atom)` - A tuple representing the function/method.
    ///   - `.0`: The FQCN of the class if it's a method, or the FQN of the function if global/namespaced.
    ///   - `.1`: The method name if it's a method, or `Atom::empty()` if it's a function.
    FunctionLike((Atom, Atom)),
}

impl std::fmt::Display for GenericParent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericParent::ClassLike(id) => write!(f, "{id}"),
            GenericParent::FunctionLike(id) => {
                let part1 = id.0;
                let part2 = id.1;

                if part1.is_empty() { write!(f, "{part2}()") } else { write!(f, "{part1}::{part2}()") }
            }
        }
    }
}
