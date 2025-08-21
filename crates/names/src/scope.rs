use std::borrow::Cow;

use ahash::HashMap;
use serde::Deserialize;
use serde::Serialize;

use mago_syntax::ast::*;

use crate::kind::NameKind;

/// Represents the scope for resolving PHP names, holding the current namespace
/// and any 'use' aliases defined within it.
///
/// This struct keeps track of the current namespace and the different types
/// of aliases (`use`, `use function`, `use const`). It provides methods to
/// manage aliases and resolve names according to PHP's rules (handling FQNs,
/// aliases, the `namespace` keyword, and namespace relativity).
///
/// Aliases are stored case-insensitively (keys in the maps are lowercase)
/// but resolve to the original case-sensitive FQN.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct NamespaceScope {
    /// The fully qualified name of the current namespace context (e.g., "App\\Http\\Controllers").
    /// `None` indicates the global namespace.
    namespace_name: Option<String>,

    /// Stores aliases for classes, interfaces, traits, and namespaces.
    /// Key: Lowercase alias name. Value: FQN.
    default_aliases: HashMap<String, String>,

    /// Stores aliases for functions.
    /// Key: Lowercase alias name. Value: FQN.
    function_aliases: HashMap<String, String>,

    /// Stores aliases for constants.
    /// Key: Lowercase alias name. Value: FQN.
    constant_aliases: HashMap<String, String>,
}

impl NamespaceScope {
    /// Creates a new, empty scope, optionally associated with a namespace.
    pub fn new(namespace_name: Option<String>) -> Self {
        NamespaceScope {
            namespace_name,
            default_aliases: HashMap::default(),
            function_aliases: HashMap::default(),
            constant_aliases: HashMap::default(),
        }
    }

    /// Creates a new, empty scope representing the global namespace.
    pub fn global() -> Self {
        NamespaceScope {
            namespace_name: None,
            default_aliases: HashMap::default(),
            function_aliases: HashMap::default(),
            constant_aliases: HashMap::default(),
        }
    }

    /// Creates a new, empty scope representing the given namespace.
    #[inline]
    pub fn for_namespace(namespace: impl Into<String>) -> Self {
        NamespaceScope {
            namespace_name: Some(namespace.into()),
            default_aliases: HashMap::default(),
            function_aliases: HashMap::default(),
            constant_aliases: HashMap::default(),
        }
    }

    /// Checks if any aliases have been defined in this scope.
    pub fn has_aliases(&self) -> bool {
        // Corrected implementation for checking if *any* alias map is non-empty
        !self.default_aliases.is_empty() || !self.function_aliases.is_empty() || !self.constant_aliases.is_empty()
    }

    /// Returns the name of the current namespace, if this scope represents one.
    pub fn namespace_name(&self) -> Option<&str> {
        self.namespace_name.as_deref()
    }

    /// Returns a reference to the map of default (class/namespace) aliases.
    pub fn default_aliases(&self) -> &HashMap<String, String> {
        &self.default_aliases
    }

    /// Returns a reference to the map of function aliases.
    pub fn function_aliases(&self) -> &HashMap<String, String> {
        &self.function_aliases
    }

    /// Returns a reference to the map of constant aliases.
    pub fn constant_aliases(&self) -> &HashMap<String, String> {
        &self.constant_aliases
    }

    /// Populates the scope's alias tables based on a PHP `use` statement AST node.
    ///
    /// This method processes the different forms of `use` statements (simple, function/const,
    /// grouped) and registers the corresponding aliases within this `NamespaceScope`.
    /// It iterates through the items declared in the `use` statement and calls `self.add`
    /// for each one with the appropriate kind, fully qualified name, and optional alias.
    ///
    /// # Arguments
    ///
    /// * `interner` - A string interner used to resolve identifiers/names from the AST nodes
    ///   into actual string (`&str`) representations.
    /// * `r#use` - A reference to the `Use` AST node representing the `use` statement.
    ///   (Parameter is named `r#use` because `use` is a Rust keyword).
    pub fn populate_from_use(&mut self, r#use: &Use<'_>) {
        match &r#use.items {
            UseItems::Sequence(use_item_sequence) => {
                for use_item in use_item_sequence.items.iter() {
                    let name = use_item.name.value().trim_start_matches("\\");
                    let alias = use_item.alias.as_ref().map(|alias_node| alias_node.identifier.value);

                    // Add as a default (class/namespace) alias
                    self.add(NameKind::Default, name, alias);
                }
            }
            UseItems::TypedSequence(typed_use_item_sequence) => {
                // Determine if it's a function or const import based on the type node
                let name_kind = match &typed_use_item_sequence.r#type {
                    UseType::Function(_) => NameKind::Function,
                    UseType::Const(_) => NameKind::Constant,
                };

                for use_item in typed_use_item_sequence.items.iter() {
                    let name = use_item.name.value().trim_start_matches("\\");
                    let alias = use_item.alias.as_ref().map(|alias_node| alias_node.identifier.value);

                    // Add with the determined kind (Function or Constant)
                    self.add(name_kind, name, alias);
                }
            }
            UseItems::TypedList(typed_use_item_list) => {
                // Determine the kind for the entire group
                let name_kind = match &typed_use_item_list.r#type {
                    UseType::Function(_) => NameKind::Function,
                    UseType::Const(_) => NameKind::Constant,
                };

                // Get the common namespace prefix for the group
                let prefix = (typed_use_item_list.namespace.value()).trim_start_matches("\\");

                for use_item in typed_use_item_list.items.iter() {
                    let name_part = use_item.name.value();
                    let alias = use_item.alias.as_ref().map(|alias_node| &alias_node.identifier.value);

                    // Construct the full FQN by combining prefix and name part
                    let fully_qualified_name = format!("{prefix}\\{name_part}");

                    // Add the alias for the fully constructed name
                    self.add(name_kind, fully_qualified_name, alias);
                }
            }
            UseItems::MixedList(mixed_use_item_list) => {
                // Get the common namespace prefix for the group
                let prefix = (mixed_use_item_list.namespace.value()).trim_start_matches("\\");

                for mixed_use_item in mixed_use_item_list.items.iter() {
                    // Determine the kind for *this specific item* within the mixed list
                    let name_kind = match &mixed_use_item.r#type {
                        None => NameKind::Default, // No type specified, defaults to class/namespace
                        Some(UseType::Function(_)) => NameKind::Function,
                        Some(UseType::Const(_)) => NameKind::Constant,
                    };

                    // Extract name/alias from the nested item structure (assuming `item` field)
                    let name_part = mixed_use_item.item.name.value();
                    let alias = mixed_use_item.item.alias.as_ref().map(|alias_node| &alias_node.identifier.value);

                    // Construct the full FQN: prefix\name_part
                    let fully_qualified_name = format!("{prefix}\\{name_part}");

                    // Add the alias with its specific kind
                    self.add(name_kind, fully_qualified_name, alias);
                }
            }
        }
    }

    /// Adds a new alias based on a 'use' statement to this scope.
    ///
    /// # Arguments
    ///
    /// * `kind` - The type of alias (`NameKind::Default`, `NameKind::Function`, `NameKind::Constant`).
    /// * `name` - The fully qualified name being imported (e.g., "App\\Models\\User").
    /// * `alias` - An optional explicit alias name (e.g., "UserModel"). If `None`, the alias
    ///   is derived from the last part of the `name` (e.g., "User" from "App\\Models\\User").
    ///
    /// The alias name (explicit or derived) is stored lowercase as the key.
    #[inline]
    pub fn add(&mut self, kind: NameKind, name: impl AsRef<str>, alias: Option<impl AsRef<str>>) {
        self.add_str(kind, name.as_ref(), alias.as_ref().map(|a| a.as_ref()))
    }

    /// non-generic version of `add` that takes a string slice.
    fn add_str(&mut self, kind: NameKind, name_ref: &str, alias: Option<&str>) {
        let alias_key = match alias {
            Some(alias) => alias.to_ascii_lowercase(),
            None => {
                if let Some(last_backslash_pos) = name_ref.rfind('\\') {
                    name_ref[last_backslash_pos + 1..].to_ascii_lowercase()
                } else {
                    name_ref.to_ascii_lowercase()
                }
            }
        };

        match kind {
            NameKind::Default => self.default_aliases.insert(alias_key, name_ref.to_owned()),
            NameKind::Function => self.function_aliases.insert(alias_key, name_ref.to_owned()),
            NameKind::Constant => self.constant_aliases.insert(alias_key, name_ref.to_owned()),
        };
    }

    /// Qualifies a simple name by prepending the current namespace, if applicable.
    ///
    /// This method is intended for simple names (containing no `\`) that are
    /// *not* explicitly aliased or handled by `\` or `namespace\` prefixes.
    /// If the current scope has a non-empty namespace name, it will be prepended.
    /// Otherwise, the original name is returned.
    ///
    /// # Arguments
    ///
    /// * `name` - The simple name to qualify (e.g., "User").
    ///
    /// # Returns
    ///
    /// The qualified name (e.g., "App\\Models\\User") or the original name if
    /// in the global scope, the namespace is empty, or the input name was not simple.
    #[inline]
    pub fn qualify_name(&self, name: impl AsRef<str>) -> String {
        self.qualify_name_str(name.as_ref()).into_owned()
    }

    /// non-generic version of `qualify_name` that takes a string slice.
    fn qualify_name_str<'a>(&self, name_ref: &'a str) -> Cow<'a, str> {
        match &self.namespace_name {
            // If we have a non-empty namespace, prepend it.
            Some(ns) if !ns.is_empty() => Cow::Owned(format!("{ns}\\{name_ref}")),
            // Otherwise (no namespace, or empty namespace), return the name as is.
            _ => Cow::Borrowed(name_ref),
        }
    }

    /// Resolves a name fully according to PHP's rules within this scope.
    ///
    /// # Arguments
    /// * `kind` - The context (`Default`, `Function`, `Constant`).
    /// * `name` - The name string to resolve.
    ///
    /// # Returns
    ///
    /// A tuple `(String, bool)`:
    ///  - The resolved or qualified name.
    ///  - `true` if resolved via explicit alias/construct (step 1), `false` otherwise.
    #[inline]
    pub fn resolve(&self, kind: NameKind, name: impl AsRef<str>) -> (String, bool) {
        let (cow, imported) = self.resolve_str(kind, name.as_ref());

        (cow.into_owned(), imported)
    }

    /// non-generic version of `resolve` that takes a string slice.
    #[inline]
    pub fn resolve_str<'a>(&self, kind: NameKind, name_ref: &'a str) -> (Cow<'a, str>, bool) {
        // Try resolving using explicit aliases and constructs
        if let Some(resolved_name) = self.resolve_alias_str(kind, name_ref) {
            return (resolved_name, true); // Resolved via alias or explicit construct
        }

        // Qualify it using the current namespace.
        (self.qualify_name_str(name_ref), false)
    }

    /// Attempts to resolve a name using *only* explicit aliases and constructs.
    ///
    /// Does *not* attempt to qualify simple names relative to the current namespace if they aren't aliased.
    ///
    /// # Arguments
    ///
    /// * `kind` - The context (`Default`, `Function`, `Constant`).
    /// * `name` - The name string to resolve.
    ///
    /// # Returns
    ///
    /// * `Some(String)` containing the resolved FQN if an explicit rule applies.
    /// * `None` if no explicit rule resolves the name.
    #[inline]
    pub fn resolve_alias(&self, kind: NameKind, name: impl AsRef<str>) -> Option<String> {
        self.resolve_alias_str(kind, name.as_ref()).map(|cow| cow.into_owned())
    }

    /// non-generic version of `resolve_alias` that takes a string slice.
    fn resolve_alias_str<'a>(&self, kind: NameKind, name_ref: &'a str) -> Option<Cow<'a, str>> {
        if name_ref.is_empty() {
            return None;
        }

        // Handle `\FQN`
        if let Some(fqn) = name_ref.strip_prefix('\\') {
            return Some(Cow::Borrowed(fqn));
        }

        let parts = name_ref.split('\\').collect::<Vec<_>>();
        let first_part = parts[0];
        let first_part_lower = first_part.to_ascii_lowercase();

        if parts.len() > 1 {
            let suffix = parts[1..].join("\\");

            // Handle `namespace\Suffix`
            if first_part_lower == "namespace" {
                match &self.namespace_name {
                    Some(namespace_prefix) => {
                        let mut resolved = namespace_prefix.clone();
                        resolved.push('\\');
                        resolved.push_str(&suffix);
                        Some(Cow::Owned(resolved))
                    }
                    None => Some(Cow::Owned(suffix)), // Relative to global "" namespace
                }
            } else {
                // Handle `Alias\Suffix`
                match self.default_aliases.get(&first_part_lower) {
                    Some(resolved_alias_fqn) => {
                        let mut resolved = resolved_alias_fqn.clone();
                        resolved.push('\\');
                        resolved.push_str(&suffix);
                        Some(Cow::Owned(resolved))
                    }
                    None => None, // Alias not found
                }
            }
        } else {
            // Handle single-part alias lookup
            (match kind {
                NameKind::Default => self.default_aliases.get(&first_part_lower).cloned(),
                NameKind::Function => self.function_aliases.get(&first_part_lower).cloned(),
                NameKind::Constant => self.constant_aliases.get(&first_part_lower).cloned(),
            })
            .map(Cow::Owned)
        }
    }
}
