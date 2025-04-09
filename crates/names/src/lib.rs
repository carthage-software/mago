use std::collections::HashSet;

use ahash::HashMap;
use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_span::HasPosition;
use mago_span::Position;
use mago_syntax::ast::Program;

use crate::resolver::NameResolver;

pub mod kind;
pub mod resolver;
pub mod scope;

mod internal;

/// Stores the results of a name resolution pass over a PHP program.
///
/// Maps source code positions (specifically, the starting byte offset of identifiers)
/// to their resolved fully qualified name (`StringIdentifier`) and a flag indicating
/// whether the resolution involved an explicit `use` alias or construct.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct ResolvedNames {
    /// Internal map storing: position (byte offset) -> (Resolved Name ID, Was Imported Flag)
    names: HashMap<usize, (StringIdentifier, bool)>,
}

impl ResolvedNames {
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
    #[deprecated = "use `NameResolver` instead."]
    pub fn resolve(interner: &ThreadedInterner, program: &Program) -> Self {
        let resolver = NameResolver::new(interner);

        resolver.resolve(program)
    }

    /// Returns the total number of resolved names stored.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Returns `true` if no resolved names are stored.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Checks if a resolved name exists for the given source `Position`.
    pub fn contains(&self, position: &Position) -> bool {
        self.names.contains_key(&position.offset)
    }

    /// Gets the resolved name identifier for the given source position.
    ///
    /// # Panics
    ///
    /// Panics if no resolved name is found at the specified `position`.
    /// Use `contains` first if unsure.
    pub fn get<T: HasPosition>(&self, position: &T) -> &StringIdentifier {
        self.names
            .get(&position.offset()) //
            .map(|(name, _)| name)
            .expect("resolved name not found at position")
    }

    /// Checks if the name resolved at the given position originated from an explicit `use` alias or construct.
    ///
    /// Returns `false` if the name was resolved relative to the namespace, is a definition,
    /// or if no name is found at the position.
    pub fn is_imported<T: HasPosition>(&self, position: &T) -> bool {
        self.names
            .get(&position.offset()) // Get Option<(StringIdentifier, bool)>
            .map(|(_, imported)| *imported) // Extract the bool flag
            .unwrap_or(false) // Default to false if position not found
    }

    /// Inserts a resolution result into the map (intended for internal use).
    ///
    /// Associates the resolved `name` identifier and its `imported` status with the
    /// given `position` (byte offset).
    pub(crate) fn insert_at<T: HasPosition>(&mut self, position: &T, name: StringIdentifier, imported: bool) {
        self.names.insert(position.offset(), (name, imported));
    }

    /// Returns a `HashSet` containing references to all stored resolution results.
    ///
    /// Each element in the set is a reference to a tuple: `(&usize, &(StringIdentifier, bool))`,
    /// representing `(&position, &(resolved_name_id, was_imported_flag))`.
    pub fn all(&self) -> HashSet<(&usize, &(StringIdentifier, bool))> {
        HashSet::from_iter(self.names.iter())
    }
}
