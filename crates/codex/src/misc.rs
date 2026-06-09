use mago_span::Span;
use mago_word::Word;
use mago_word::concat_word;

/// Represents a PHP variable identifier (e.g., `$foo`, `$this`).
/// Wraps a `Word` which holds the interned name (including '$').
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VariableIdentifier(
    /// The atom for the variable name (e.g., "$foo").
    pub Word,
);

/// Identifies the target of an expression, distinguishing simple variables from property accesses.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExpressionIdentifier {
    /// A simple variable identifier.
    ///
    /// * `VariableIdentifier` - The identifier for the variable (e.g., `$foo`).
    Variable(VariableIdentifier),
    /// An instance property access (e.g., `$this->prop`, `$user->name`).
    ///
    /// * `VariableIdentifier` - The identifier for the object variable (e.g., `$this`, `$user`).
    /// * `Span` - The source code location covering the property name part (e.g., `prop` or `name`).
    /// * `Word` - The name of the property being accessed (e.g., `prop`, `name`).
    InstanceProperty(VariableIdentifier, Span, Word),
}

/// Identifies the scope where a generic template parameter (`@template`) is defined.
#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GenericParent {
    /// The template is defined on a class, interface, trait, or enum.
    /// * `Word` - The fully qualified name (FQCN) of the class-like structure.
    ClassLike(Word),
    /// The template is defined on a function or method.
    /// * `(Word, Word)` - A tuple representing the function/method.
    ///   - `.0`: The FQCN of the class if it's a method, or the FQN of the function if global/namespaced.
    ///   - `.1`: The method name if it's a method, or `Word::empty()` if it's a function.
    FunctionLike((Word, Word)),
}

impl GenericParent {
    /// Builds the identifier form of this scope using the raw interned bytes of every `Word`.
    ///
    /// Unlike the [`Display`](std::fmt::Display) impl, this never escapes, so a non-UTF-8
    /// class or function name round-trips byte-for-byte into the resulting id. This matters
    /// because type ids key the subtype cache and combiner dedup, where an escaped rendering
    /// could collide distinct byte sequences.
    #[inline]
    #[must_use]
    pub fn id_word(&self) -> Word {
        match self {
            GenericParent::ClassLike(id) => *id,
            GenericParent::FunctionLike((part1, part2)) => {
                if part1.is_empty() {
                    concat_word!(part2.as_bytes(), b"()")
                } else {
                    concat_word!(part1.as_bytes(), b"::", part2.as_bytes(), b"()")
                }
            }
        }
    }
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
