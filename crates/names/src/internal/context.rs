use mago_allocator::prelude::*;

use mago_syntax::ast::Use;

use crate::kind::NameKind;
use crate::scope::NamespaceScope;

/// Maintains the current name resolution state during an AST walk.
///
/// This struct acts as a stateful manager for the name resolution process, primarily
/// by holding the current `NamespaceScope` (which contains the active namespace name
/// and relevant `use` aliases).
///
/// It serves as a bridge between the AST walker and the `NamespaceScope`.
#[derive(Debug)]
pub struct NameResolutionContext<'arena, A>
where
    A: Arena,
{
    arena: &'arena A,
    scope: NamespaceScope,
}

impl<'arena, A> NameResolutionContext<'arena, A>
where
    A: Arena,
{
    /// Creates a new `NameResolutionContext`, initialized to the global namespace scope.
    ///
    /// # Arguments
    ///
    /// * `arena` - A reference to the arena used for memory allocation.
    pub fn new(arena: &'arena A) -> Self {
        NameResolutionContext {
            arena,
            // Start in the global scope by default.
            scope: NamespaceScope::global(),
        }
    }

    /// Updates the current scope to reflect entering a PHP namespace declaration.
    ///
    /// This replaces the existing internal `NamespaceScope` with a new one configured
    /// for the specified namespace.
    ///
    /// # Arguments
    ///
    /// * `namespace` - An `Option<&StringIdentifier>` representing the declared namespace name.
    ///   - `Some(id)`: Enters the namespace identified by `id`.
    ///   - `None`: Enters the global namespace (e.g., from `namespace;`).
    pub fn enter_namespace(&mut self, namespace_name: Option<&[u8]>) {
        match namespace_name {
            Some(namespace_name) => {
                // Create a new scope specific to this namespace.
                self.scope = NamespaceScope::for_namespace(namespace_name.to_vec());
            }
            None => {
                // Reset to a fresh global scope.
                self.scope = NamespaceScope::global();
            }
        }
    }

    /// Resets the current scope back to the global namespace scope.
    pub fn exit_namespace(&mut self) {
        self.scope = NamespaceScope::global();
    }

    /// Processes a `use` statement AST node, adding its aliases to the current scope.
    ///
    /// Delegates directly to the underlying `NamespaceScope`'s `populate_from_use` method,
    /// passing the required interner reference along with the `Use` node.
    ///
    /// # Arguments
    ///
    /// * `r#use` - The `Use` AST node to process.
    pub fn populate_from_use(&mut self, r#use: &Use) {
        self.scope.populate_from_use(r#use);
    }

    /// Qualifies a simple name identifier relative to the current namespace scope.
    ///
    /// # Arguments
    ///
    /// * `name` - The `StringIdentifier` of the simple name to qualify.
    ///
    /// # Returns
    ///
    /// The `StringIdentifier` for the potentially qualified name.
    pub fn qualify_name(&self, name: &[u8]) -> &'arena [u8] {
        let qualified_str = self.scope.qualify_name(name);

        self.arena.alloc_slice_copy(&qualified_str)
    }

    /// Allocates `s` in the arena and returns a borrow with the arena lifetime.
    pub fn intern(&self, s: &[u8]) -> &'arena [u8] {
        self.arena.alloc_slice_copy(s)
    }

    /// Performs full name resolution for a given identifier within the current scope.
    ///
    /// # Arguments
    ///
    /// * `kind` - The `NameKind` (Default, Function, Constant) indicating the context.
    ///
    /// # Returns
    ///
    /// A tuple `(StringIdentifier, bool)` where:
    ///  - The `StringIdentifier` represents the resolved fully qualified name.
    ///  - The `bool` is `true` if resolution occurred via an explicit alias or construct
    ///    (like `\` or `namespace\`), and `false` otherwise (e.g., resolved relative
    ///    to the namespace or returned as-is).
    pub fn resolve<'name>(&self, kind: NameKind, name_str: &'name [u8]) -> (&'arena [u8], bool) {
        let (cow, is_imported) = self.scope.resolve_str(kind, name_str);

        (self.arena.alloc_slice_copy(&cow), is_imported)
    }
}
