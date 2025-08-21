use bumpalo::Bump;
use mago_collector::Collector;
use mago_database::file::File;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_span::HasPosition;

use crate::scope::ScopeStack;

#[derive(Debug)]
pub struct LintContext<'input, 'ast, 'arena> {
    pub php_version: PHPVersion,
    pub arena: &'arena Bump,
    pub source_file: &'input File,
    pub resolved_names: &'ast ResolvedNames<'arena>,
    pub collector: Collector<'input, 'arena>,
    pub scope: ScopeStack<'arena>,
}

impl<'input, 'ast, 'arena> LintContext<'input, 'ast, 'arena> {
    pub fn new(
        php_version: PHPVersion,
        arena: &'arena Bump,
        source_file: &'input File,
        resolved_names: &'ast ResolvedNames<'arena>,
        collector: Collector<'input, 'arena>,
    ) -> Self {
        Self { php_version, arena, source_file, resolved_names, collector, scope: ScopeStack::new() }
    }

    /// Checks if a name at a given position is imported.
    pub fn is_name_imported(&self, position: &impl HasPosition) -> bool {
        self.resolved_names.is_imported(&position.position())
    }

    /// Retrieves the name associated with a given position in the code.
    ///
    /// # Panics
    ///
    /// Panics if no name is found at the specified position.
    pub fn lookup_name(&self, position: &impl HasPosition) -> &'arena str {
        self.resolved_names.get(&position.position())
    }
}
