use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
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
pub struct NameResolutionContext<'a> {
    interner: &'a ThreadedInterner,
    scope: NamespaceScope,
}

impl<'a> NameResolutionContext<'a> {
    /// Creates a new `NameResolutionContext`, initialized to the global namespace scope.
    ///
    /// # Arguments
    ///
    /// * `interner` - A reference to the `ThreadedInterner` to be used throughout
    ///   the context's lifetime.
    pub fn new(interner: &'a ThreadedInterner) -> Self {
        NameResolutionContext {
            interner,
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
    pub fn enter_namespace(&mut self, namespace: Option<&StringIdentifier>) {
        match namespace {
            Some(namespace_id) => {
                // Look up the string name for the namespace ID.
                let namespace_name = self.interner.lookup(namespace_id);
                // Create a new scope specific to this namespace.
                self.scope = NamespaceScope::for_namespace(namespace_name);
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
        self.scope.populate_from_use(self.interner, r#use);
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
    pub fn qualify_name(&self, name: &StringIdentifier) -> StringIdentifier {
        // Changed to &self
        // Convert ID to string
        let name_str = self.interner.lookup(name);
        // Qualify the string using the current scope's logic
        let qualified_str = self.scope.qualify_name(name_str);
        // Convert the resulting string back to an ID
        self.interner.intern(qualified_str)
    }

    /// Performs full name resolution for a given identifier within the current scope.
    ///
    /// # Arguments
    ///
    /// * `kind` - The `NameKind` (Default, Function, Constant) indicating the context.
    /// * `name_id` - The `StringIdentifier` of the name to resolve.
    ///
    /// # Returns
    ///
    /// A tuple `(StringIdentifier, bool)` where:
    ///  - The `StringIdentifier` represents the resolved fully qualified name.
    ///  - The `bool` is `true` if resolution occurred via an explicit alias or construct
    ///    (like `\` or `namespace\`), and `false` otherwise (e.g., resolved relative
    ///    to the namespace or returned as-is).
    pub fn resolve(&self, kind: NameKind, name_id: &StringIdentifier) -> (StringIdentifier, bool) {
        // Convert input ID to string
        let name_str = self.interner.lookup(name_id);

        // Resolve the string using the current scope's full resolution logic
        let (resolved_name_str, is_imported) = self.scope.resolve(kind, name_str);

        // Convert the resulting resolved string back to an ID
        let resolved_name_id = self.interner.intern(resolved_name_str);

        (resolved_name_id, is_imported)
    }
}
