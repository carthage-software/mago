use std::collections::HashSet;

use ahash::HashMap;
use serde::Deserialize;
use serde::Serialize;

use fennec_ast::Program;
use fennec_interner::StringIdentifier;
use fennec_interner::ThreadedInterner;
use fennec_span::HasPosition;
use fennec_span::Position;
use fennec_walker::MutWalker;

use crate::internal::context::NameContext;
use crate::internal::resolver::NameResolver;

mod internal;

/// Represents a collection of resolved names in a program.
///
/// This struct stores a mapping of positions (represented as byte offsets)
/// to resolved names (represented as `StringIdentifier`s).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Names {
    names: HashMap<usize, (StringIdentifier, bool)>,
}

impl Names {
    /// Resolves names in the given program.
    ///
    /// This method traverses the AST of the program, resolves names using a `NameResolver` and `NameContext`,
    /// and returns a `Names` instance containing the resolved names.
    ///
    /// # Arguments
    ///
    /// * `interner` - A `ThreadedInterner` used for string interning.
    /// * `program` - A reference to the `Program` AST node.
    ///
    /// # Returns
    ///
    /// A `Names` instance containing the resolved names.
    pub fn resolve(interner: &ThreadedInterner, program: &Program) -> Self {
        let mut resolver: NameResolver = NameResolver::new();
        let mut context: NameContext = NameContext::new(interner);

        resolver.walk_program(program, &mut context);

        resolver.resolved_names
    }

    /// Returns the number of resolved names.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Returns `true` if there are no resolved names.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Checks if a name is resolved at the given position.
    ///
    /// # Arguments
    ///
    /// * `position` - A reference to the `Position` in the code.
    ///
    /// # Returns
    ///
    /// `true` if a name is resolved at the given position, `false` otherwise.
    pub fn contains(&self, position: &Position) -> bool {
        self.names.contains_key(&position.offset)
    }

    /// Gets the resolved name at the given position.
    ///
    /// # Arguments
    ///
    /// * `position` - A reference to a type that implements `HasPosition`.
    ///
    /// # Returns
    ///
    /// The `StringIdentifier` of the resolved name.
    ///
    /// # Panics
    ///
    /// Panics if the name is not found at the given position.
    pub fn get(&self, position: &impl HasPosition) -> &StringIdentifier {
        return self.names.get(&position.position().offset).map(|(name, _)| name).expect("name not found at position");
    }

    /// Returns whether the name at the given position was explicitly imported.
    ///
    /// # Arguments
    ///
    /// * `position` - A reference to the `Position` in the code.
    ///
    /// # Returns
    ///
    /// `true` if the name was imported, `false` otherwise.
    pub fn is_imported(&self, position: &impl HasPosition) -> bool {
        return self.names.get(&position.position().offset).map(|(_, imported)| *imported).unwrap_or(false);
    }

    /// Inserts a resolved name at the given position.
    ///
    /// This method is intended for internal use within the crate.
    ///
    /// # Arguments
    ///
    /// * `position` - The position (as a byte offset) where the name is resolved.
    /// * `name` - The `StringIdentifier` of the resolved name.
    pub(crate) fn insert_at<P: Into<usize>>(&mut self, position: P, name: StringIdentifier, imported: bool) {
        self.names.insert(position.into(), (name, imported));
    }

    /// Returns a set of all resolved names.
    ///
    /// The set contains tuples of positions and resolved names.
    pub fn all(&self) -> HashSet<(&usize, &(StringIdentifier, bool))> {
        HashSet::from_iter(self.names.iter())
    }
}
