use mago_codex::metadata::CodebaseMetadata;
use mago_names::ResolvedNames;
use mago_span::HasPosition;

use crate::report::FortressReport;
use crate::report::breach::BoundaryBreach;
use crate::report::flaw::StructuralFlaw;
use crate::settings::Settings;

/// Context for guard operations, providing access to resolved names and issue collection.
#[derive(Debug)]
pub struct GuardContext<'ctx, 'arena> {
    pub resolved_names: &'ctx ResolvedNames<'arena>,
    pub settings: &'ctx Settings,
    pub codebase: &'ctx CodebaseMetadata,
    pub boundary_breaches: Vec<BoundaryBreach>,
    pub structural_flaws: Vec<StructuralFlaw>,
    pub current_namespace: Option<&'arena str>,
}

impl<'ctx, 'arena> GuardContext<'ctx, 'arena> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        resolved_names: &'ctx ResolvedNames<'arena>,
        settings: &'ctx Settings,
        codebase: &'ctx CodebaseMetadata,
    ) -> Self {
        Self {
            resolved_names,
            settings,
            codebase,
            boundary_breaches: vec![],
            structural_flaws: vec![],
            current_namespace: None,
        }
    }

    /// Sets the current namespace in the context.
    pub fn set_current_namespace(&mut self, namespace: Option<&'arena str>) {
        self.current_namespace = namespace;
    }

    /// Gets the current namespace from the context, or an empty string if none is set.
    pub fn get_current_namespace(&self) -> &'arena str {
        self.current_namespace.unwrap_or("")
    }

    /// Retrieves the fully qualified name associated with a given position in the code.
    ///
    /// # Panics
    ///
    /// Panics if no name is found at the specified position.
    pub fn lookup_name(&self, position: &impl HasPosition) -> &'arena str {
        self.resolved_names.get(&position.position())
    }

    /// Attempts to retrieve the fully qualified name associated with a given position.
    ///
    /// Returns `None` if no name is found at the specified position.
    pub fn try_lookup_name(&self, position: &impl HasPosition) -> Option<&'arena str> {
        let pos = position.position();
        if self.resolved_names.contains(&pos) { Some(self.resolved_names.get(&pos)) } else { None }
    }

    /// Consumes the context and generates a `FortressReport` containing all collected issues.
    pub fn report(self) -> FortressReport {
        FortressReport { boundary_breaches: self.boundary_breaches, structural_flaws: self.structural_flaws }
    }
}
