use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;

use crate::misc::GenericParent;
use crate::ttype::union::TUnion;

/// Holds contextual information necessary for resolving generic template types (`@template`).
///
/// This context typically includes the definitions of template parameters available in the current scope
/// (e.g., from class or function `@template` tags) and any concrete types that these templates
/// have been resolved to (e.g., when a generic class is instantiated or a generic method is called).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeResolutionContext {
    /// Definitions of template types available in this context, including their constraints.
    template_definitions: Vec<(Atom, Vec<(GenericParent, TUnion)>)>,

    /// Concrete types that template parameters (often from an outer scope) resolve to
    /// within this specific context.
    resolved_template_types: Vec<(Atom, TUnion)>,
}

/// Provides a default, empty type resolution context.
impl Default for TypeResolutionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeResolutionContext {
    /// Creates a new, empty `TypeResolutionContext` with no defined or resolved template types.
    pub fn new() -> Self {
        Self { template_definitions: vec![], resolved_template_types: vec![] }
    }

    /// Checks if this context is empty, meaning it has no template definitions or resolved types.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.template_definitions.is_empty() && self.resolved_template_types.is_empty()
    }

    /// Adds a template type definition (e.g., from an `@template T of Constraint` tag).
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the template parameter (e.g., `"T"`).
    /// * `constraints`: A list of constraints, each specifying the origin (parent) and the constraint type.
    pub fn with_template_definition(mut self, name: Atom, constraints: Vec<(GenericParent, TUnion)>) -> Self {
        self.template_definitions.push((name, constraints));
        self
    }

    /// Adds a mapping indicating that a template parameter resolves to a specific concrete type
    /// within this context.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the template parameter (e.g., `"T"`).
    /// * `resolved_type`: The concrete `TUnion` type that `name` resolves to here.
    pub fn with_resolved_template_type(mut self, name: Atom, resolved_type: TUnion) -> Self {
        self.resolved_template_types.push((name, resolved_type));
        self
    }

    /// Returns a slice of the defined template parameters and their constraints for this context.
    #[inline]
    pub fn get_template_definitions(&self) -> &[(Atom, Vec<(GenericParent, TUnion)>)] {
        &self.template_definitions
    }

    /// Returns a mutable slice of the defined template parameters and their constraints for this context.
    #[inline]
    pub fn get_template_definitions_mut(&mut self) -> &mut [(Atom, Vec<(GenericParent, TUnion)>)] {
        &mut self.template_definitions
    }

    /// Returns a slice of the template parameters that have resolved to concrete types in this context.
    #[inline]
    pub fn get_resolved_template_types(&self) -> &[(Atom, TUnion)] {
        &self.resolved_template_types
    }

    /// Returns a mutable slice of the template parameters that have resolved to concrete types in this context.
    #[inline]
    pub fn get_resolved_template_types_mut(&mut self) -> &mut [(Atom, TUnion)] {
        &mut self.resolved_template_types
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
    pub fn get_template_definition(&self, name: &str) -> Option<&Vec<(GenericParent, TUnion)>> {
        self.template_definitions.iter().find(|(n, _)| n == name).map(|(_, constraints)| constraints)
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
    pub fn has_template_definition(&self, name: &str) -> bool {
        self.template_definitions.iter().any(|(n, _)| n == name)
    }

    /// Looks up the concrete type that a specific template parameter resolves to in this context.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the template parameter (e.g., `"T"`) to look up.
    ///
    /// # Returns
    ///
    /// `Some` containing a reference to the resolved `TUnion` type if found, `None` otherwise.
    /// Note: If multiple entries exist for the same name (due to shadowing or errors),
    /// this currently returns the first match found.
    pub fn get_resolved_template_type(&self, name: &str) -> Option<&TUnion> {
        self.resolved_template_types
            .iter()
            // Iterate in reverse if shadowing means the *last* added binding is correct
            // .rev()
            .find(|(n, _)| n == name)
            .map(|(_, resolved_type)| resolved_type)
    }

    /// Checks if this context contains any template definitions or resolved template types.
    #[inline]
    pub fn has_templates(&self) -> bool {
        !self.template_definitions.is_empty() || !self.resolved_template_types.is_empty()
    }

    /// Checks if a specific template parameter has a concrete resolved type in this context.
    #[inline]
    pub fn is_template_resolved(&self, name: &str) -> bool {
        self.resolved_template_types.iter().any(|(n, _)| n == name)
    }

    /// Merges another `TypeResolutionContext` into this one, combining their template definitions
    /// and resolved types.
    #[inline]
    pub fn merge(&mut self, other: TypeResolutionContext) {
        self.template_definitions.extend(other.template_definitions);
        self.resolved_template_types.extend(other.resolved_template_types);
    }
}
