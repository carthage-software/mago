use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;

use crate::ttype::template::GenericTemplate;

/// Holds contextual information necessary for resolving generic template types (`@template`).
///
/// This context typically includes the definitions of template parameters available in the current scope
/// (e.g., from class or function `@template` tags) and any concrete types that these templates
/// have been resolved to (e.g., when a generic class is instantiated or a generic method is called).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeResolutionContext {
    /// Definitions of template types available in this context, including their constraints.
    template_definitions: WordMap<Vec<GenericTemplate>>,

    /// Type aliases defined in the current class scope (from @type tags).
    type_aliases: WordSet,

    /// Imported type aliases (from @import-type tags).
    /// Maps local alias name to (source class FQCN, original type name).
    imported_type_aliases: WordMap<(Word, Word)>,
}

/// Provides a default, empty type resolution context.
impl Default for TypeResolutionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeResolutionContext {
    /// Creates a new, empty `TypeResolutionContext` with no defined or resolved template types.
    #[must_use]
    pub fn new() -> Self {
        Self {
            template_definitions: WordMap::default(),
            type_aliases: WordSet::default(),
            imported_type_aliases: WordMap::default(),
        }
    }

    /// Checks if this context is empty, meaning it has no template definitions or resolved types.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.template_definitions.is_empty() && self.type_aliases.is_empty() && self.imported_type_aliases.is_empty()
    }

    /// Adds a template type definition (e.g., from an `@template T of Constraint` tag).
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the template parameter (e.g., `"T"`).
    /// * `constraints`: A list of constraints for the template parameter.
    #[must_use]
    pub fn with_template_definition(mut self, name: Word, constraints: Vec<GenericTemplate>) -> Self {
        self.template_definitions.insert(name, constraints);
        self
    }

    /// Returns a mutable reference to the template definitions map.
    #[inline]
    pub fn get_template_definitions_mut(&mut self) -> &mut WordMap<Vec<GenericTemplate>> {
        &mut self.template_definitions
    }

    /// Looks up the constraints for a specific template parameter defined in this context.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the template parameter (e.g., `"T"`) to look up.
    ///
    /// # Returns
    ///
    /// `Some` containing a reference to the vector of constraints if the template is defined, `None` otherwise.
    #[must_use]
    pub fn get_template_definition(&self, name: Word) -> Option<&Vec<GenericTemplate>> {
        self.template_definitions.get(&name)
    }

    /// Checks if a specific template parameter is defined in this context.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the template parameter (e.g., `"T"`) to check.
    ///
    /// # Returns
    ///
    /// `true` if the template parameter is defined, `false` otherwise.
    #[must_use]
    pub fn has_template_definition(&self, name: Word) -> bool {
        self.template_definitions.contains_key(&name)
    }

    /// Adds type aliases from a class to this context.
    ///
    /// # Arguments
    ///
    /// * `aliases`: A set of type alias names.
    #[must_use]
    pub fn with_type_aliases(mut self, aliases: WordSet) -> Self {
        self.type_aliases = aliases;
        self
    }

    /// Adds a single type alias to this context.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the type alias to add.
    #[must_use]
    pub fn with_type_alias(mut self, name: Word) -> Self {
        self.type_aliases.insert(name);
        self
    }

    /// Checks if a specific type alias is defined in this context.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the type alias to check.
    #[must_use]
    pub fn has_type_alias(&self, name: Word) -> bool {
        self.type_aliases.contains(&name)
    }

    /// Adds an imported type alias to this context.
    ///
    /// # Arguments
    ///
    /// * `local_name`: The local name of the imported alias (possibly renamed with "as").
    /// * `source_class`: The FQCN of the class where the type alias is defined.
    /// * `original_name`: The original name of the type alias in the source class.
    #[must_use]
    pub fn with_imported_type_alias(mut self, local_name: Word, source_class: Word, original_name: Word) -> Self {
        self.imported_type_aliases.insert(local_name, (source_class, original_name));
        self
    }

    /// Looks up an imported type alias in this context.
    ///
    /// # Arguments
    ///
    /// * `name`: The local name of the imported alias to look up.
    ///
    /// # Returns
    ///
    /// `Some` containing a reference to (`source_class`, `original_name`) if found, `None` otherwise.
    #[must_use]
    pub fn get_imported_type_alias(&self, name: Word) -> Option<&(Word, Word)> {
        self.imported_type_aliases.get(&name)
    }
}
