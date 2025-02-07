use mago_php_version::PHPVersion;
use mago_reporting::Annotation;
use mago_reporting::AnnotationKind;
use mago_span::HasSpan;
use toml::value::Value;

use mago_ast::Hint;
use mago_ast::Identifier;
use mago_fixer::FixPlan;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_reflection::CodebaseReflection;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_semantics::Semantics;
use mago_span::HasPosition;

use crate::ast::AstNode;
use crate::directive::LintDirective;
use crate::ignore::IgnoreDirective;
use crate::rule::ConfiguredRule;

#[derive(Debug)]
pub struct LintContext<'a> {
    pub php_version: PHPVersion,
    pub rule: &'a ConfiguredRule,
    pub interner: &'a ThreadedInterner,
    pub codebase: &'a CodebaseReflection,
    pub semantics: &'a Semantics,
    pub ignores: Vec<&'a IgnoreDirective<'a>>,

    unused_ignores: Vec<&'a IgnoreDirective<'a>>,
    issues: Vec<Issue>,
}

impl LintContext<'_> {
    pub fn new<'a>(
        php_version: PHPVersion,
        rule: &'a ConfiguredRule,
        interner: &'a ThreadedInterner,
        codebase: &'a CodebaseReflection,
        semantics: &'a Semantics,
        ignores: Vec<&'a IgnoreDirective<'a>>,
    ) -> LintContext<'a> {
        LintContext {
            php_version,
            rule,
            interner,
            codebase,
            semantics,
            ignores,
            unused_ignores: Vec::new(),
            issues: Vec::new(),
        }
    }

    /// Determines the effective reporting level for a linter rule.
    pub fn level(&self) -> Level {
        self.rule.level
    }

    /// Retrieves the value of a rule-specific option.
    pub fn option(&self, option_name: &'static str) -> Option<&Value> {
        self.rule.settings.get_option(option_name)
    }

    /// Retrieves the string associated with a given identifier.
    ///
    /// # Panics
    ///
    /// Panics if the identifier is not found in the interner.
    pub fn lookup(&self, id: &StringIdentifier) -> &str {
        self.interner.lookup(id)
    }

    /// Checks if a name at a given position is imported.
    pub fn is_name_imported(&self, position: &impl HasPosition) -> bool {
        self.semantics.names.is_imported(&position.position())
    }

    /// Retrieves the name associated with a given position in the code.
    ///
    /// # Panics
    ///
    /// Panics if no name is found at the specified position.
    pub fn lookup_name(&self, position: &impl HasPosition) -> &str {
        let name_id = self.semantics.names.get(&position.position());

        self.lookup(name_id)
    }

    /// Resolves the correct function name based on PHP's dynamic name resolution rules.
    ///
    /// This function determines the fully qualified name (FQN) of a function being called,
    /// accounting for PHP's nuanced resolution rules:
    ///
    /// - If the function is explicitly imported via `use`, it resolves to the imported name.
    /// - If the function name starts with a leading `\`, it is treated as a global function.
    /// - If no `\` is present:
    ///   1. The function name is checked in the current namespace.
    ///   2. If not found, it falls back to the global namespace.
    ///   3. If neither exists, it defaults to the current namespace's FQN.
    ///
    /// # Arguments
    ///
    /// - `identifier`: The identifier representing the function name in the source code.
    ///
    /// # Returns
    ///
    /// - A reference to the resolved function name as a string.
    ///
    /// # Note
    ///
    /// Function names in PHP are case-insensitive; they are stored and looked up in lowercase
    /// within the codebase reflection.
    pub fn resolve_function_name(&self, identifier: &Identifier) -> &str {
        // Check if the function name is explicitly imported via `use` statement.
        // If it is, directly resolve it to the imported name.
        if self.is_name_imported(identifier) {
            return self.lookup_name(identifier);
        }

        // Retrieve the raw name identifier for the function.
        let name_id = identifier.value();
        let name = self.lookup(&name_id);

        // Check if the name explicitly refers to the global namespace using a leading `\`.
        if let Some(stripped) = name.strip_prefix('\\') {
            // If yes, return the global function name without the `\`.
            return stripped;
        }

        // If no leading `\`, resolve based on the namespace hierarchy:
        // 1. Check if the fully qualified function name (FQFN) exists in the current context.
        let fqfn_id = self.semantics.names.get(&identifier.position());
        if self.codebase.function_exists(self.interner, fqfn_id) {
            // The FQFN exists, so return it.
            return self.lookup(fqfn_id);
        }

        // If FQFN doesn't exist, check if the global function name exists.
        if !name.contains('\\') && self.codebase.function_exists(self.interner, &name_id) {
            // If global function name exists, return it.
            return name;
        }

        // If neither exists, assume the FQFN and return it.
        self.lookup(fqfn_id)
    }

    /// Resolves the correct constant name based on PHP's dynamic name resolution rules.
    ///
    /// This function determines the fully qualified name (FQN) of a constant being referenced,
    /// accounting for PHP's nuanced resolution rules:
    ///
    /// - If the constant is explicitly imported via `use const`, it resolves to the imported name.
    /// - If the constant name starts with a leading `\`, it is treated as a global constant.
    /// - If no `\` is present:
    ///   1. The constant name is checked in the current namespace.
    ///   2. If not found, it falls back to the global namespace.
    ///   3. If neither exists, it defaults to the current namespace's FQN.
    ///
    /// # Arguments
    ///
    /// - `identifier`: The identifier representing the constant name in the source code.
    ///
    /// # Returns
    ///
    /// - A reference to the resolved constant name as a string.
    ///
    /// # Note
    ///
    /// Constant names in PHP are case-sensitive, so exact matches are performed without lowering.
    pub fn resolve_constant_name(&self, identifier: &Identifier) -> &str {
        // Check if the constant name is explicitly imported via `use` statement.
        // If it is, directly resolve it to the imported name.
        if self.is_name_imported(identifier) {
            return self.lookup_name(identifier);
        }

        // Retrieve the raw name identifier for the constant.
        let name_id = identifier.value();
        let name = self.lookup(&name_id);

        // Check if the name explicitly refers to the global namespace using a leading `\`.
        if let Some(stripped) = name.strip_prefix('\\') {
            // If yes, return the global constant name without the `\`.
            return stripped;
        }

        // If no leading `\`, resolve based on the namespace hierarchy:
        // 1. Check if the fully qualified constant name (FQCN) exists in the current context.
        let fqcn_id = self.semantics.names.get(&identifier.position());
        if self.codebase.constant_exists(self.interner, fqcn_id) {
            // The FQCN exists, so return it.
            return self.lookup(fqcn_id);
        }

        // If FQCN doesn't exist, check if the global constant name exists.
        if !name.contains('\\') && self.codebase.constant_exists(self.interner, &name_id) {
            // If global constant name exists,
            return name;
        }

        // If neither exists, assume the FQCN and return it.
        self.lookup(fqcn_id)
    }

    /// Converts a type hint into a human-readable string representation.
    ///
    /// This function takes a type hint (e.g., an identifier, nullable type, union type)
    /// and resolves it into a string that can be used in error messages or similar contexts.
    /// The return value is not guaranteed to match the exact type representation in the code
    /// or in the internal reflection system—it is specifically formatted for readability.
    ///
    /// # Arguments
    ///
    /// - `hint`: The type hint to resolve. The hint can represent various constructs such as
    ///   identifiers, nullable types, unions, intersections, and keywords like `void`, `self`, etc.
    ///
    /// # Returns
    ///
    /// - A `String` containing a human-readable representation of the type hint.
    ///
    /// # Notes
    ///
    /// - Identifiers are resolved using `lookup_name` for user-defined types or symbols.
    /// - Constructs like nullable types (`?`), unions (`|`), and intersections (`&`) are recursively resolved.
    /// - The output is designed for clarity in messages (e.g., issues) rather than reflecting the exact
    ///   code structure or type definitions.
    ///
    /// # Examples
    ///
    /// - For a nullable type `?Foo`, the function returns `"?Foo"`.
    /// - For a union type `int|string`, the function returns `"int|string"`.
    /// - For a complex type like `(int|float)&string`, the function returns `"(int|float)&string"`.
    pub fn get_readable_hint(&self, hint: &Hint) -> String {
        match hint {
            Hint::Identifier(identifier) => self.lookup_name(identifier).to_string(),
            Hint::Parenthesized(parenthesized_hint) => {
                format!("({})", self.get_readable_hint(&parenthesized_hint.hint))
            }
            Hint::Nullable(nullable_hint) => format!("?{}", self.get_readable_hint(&nullable_hint.hint)),
            Hint::Union(union_hint) => {
                format!("{}|{}", self.get_readable_hint(&union_hint.left), self.get_readable_hint(&union_hint.right))
            }
            Hint::Intersection(intersection_hint) => {
                format!(
                    "{}&{}",
                    self.get_readable_hint(&intersection_hint.left),
                    self.get_readable_hint(&intersection_hint.right)
                )
            }
            Hint::Null(keyword)
            | Hint::True(keyword)
            | Hint::False(keyword)
            | Hint::Array(keyword)
            | Hint::Callable(keyword)
            | Hint::Static(keyword)
            | Hint::Self_(keyword)
            | Hint::Parent(keyword) => self.lookup(&keyword.value).to_string(),
            Hint::Void(identifier)
            | Hint::Never(identifier)
            | Hint::Float(identifier)
            | Hint::Bool(identifier)
            | Hint::Integer(identifier)
            | Hint::String(identifier)
            | Hint::Object(identifier)
            | Hint::Mixed(identifier)
            | Hint::Iterable(identifier) => self.lookup(&identifier.value).to_string(),
        }
    }

    /// Checks if the given node should be ignored based on active ignore directives.
    ///
    /// This method examines the node's starting line (using the source's precomputed line numbers)
    /// and removes (consumes) from the active ignore pool any directives that appear before the node.
    /// The first applicable ignore is used to suppress the node (causing the method to return `true`).
    /// Any further applicable ignores are moved to the `unused_ignores` vector for later reporting.
    /// Non-applicable ignores are returned to the active pool.
    ///
    /// # Parameters
    ///
    /// - `node`: A reference to the AST node (implementing [`HasSpan`]) to check against.
    ///
    /// # Returns
    ///
    /// Returns `true` if at least one ignore directive was applied (i.e. the node should be skipped),
    /// and `false` otherwise.
    #[inline]
    fn ignores(&mut self, node: impl HasSpan) -> bool {
        let node_start_line = self.semantics.source.line_number(node.span().start.offset);
        let mut ignore_applied = false;
        // Pre-allocate capacity to avoid reallocations.
        let mut remaining = Vec::with_capacity(self.ignores.len());

        for ignore in self.ignores.drain(..) {
            let applies = if ignore.own_line {
                ignore.start_line < node_start_line
            } else {
                ignore.start_line == node_start_line || ignore.end_line == node_start_line
            };

            if applies {
                if !ignore_applied {
                    // Use the first applicable ignore to suppress this node.
                    ignore_applied = true;
                } else {
                    // Additional applicable ignores are recorded as unused.
                    self.unused_ignores.push(ignore);
                }
            } else {
                // Retain ignores that do not apply.
                remaining.push(ignore);
            }
        }

        self.ignores = remaining;

        ignore_applied
    }

    /// Immediately reports the provided issue without performing any ignore checks.
    ///
    /// This method augments the issue with the current rule's slug (used as its code) and appends it to
    /// the issue collection.
    ///
    /// # Parameters
    ///
    /// - `issue`: The issue to be reported.
    #[inline(always)]
    pub fn force_report(&mut self, issue: Issue) {
        self.issues.push(issue.with_code(&self.rule.slug));
    }

    /// Reports an issue if it is not suppressed by an applicable ignore directive.
    ///
    /// This method inspects the issue's annotations for a primary annotation. If found, it checks the
    /// corresponding span against active ignore directives via `ignores()`. If an applicable ignore is found,
    /// the issue is suppressed and the method returns `false`. Otherwise, the issue is reported via
    /// `force_report()` and the method returns `true`.
    ///
    /// # Parameters
    ///
    /// - `issue`: The issue to be reported.
    ///
    /// # Returns
    ///
    /// Returns `true` if the issue was reported; `false` if it was suppressed.
    #[inline]
    pub fn report(&mut self, issue: Issue) -> bool {
        let mut span = None;
        for annotation in issue.annotations.iter() {
            if let AnnotationKind::Primary = annotation.kind {
                span = Some(annotation.span);
                break;
            }
        }

        if let Some(span) = span {
            if self.ignores(span) {
                return false;
            }
        }

        self.force_report(issue);
        true
    }

    /// Reports an issue along with a fix suggestion.
    ///
    /// This method creates a new fix plan and passes it to the provided closure `f` to configure the
    /// suggested fix. The fix is then attached to the issue (using the source identifier) and the issue
    /// is reported via `report()`. It returns `true` if the issue was reported, or `false` if it was suppressed.
    ///
    /// # Parameters
    ///
    /// - `issue`: The issue to be reported.
    /// - `f`: A closure that accepts a mutable reference to a [`FixPlan`] to configure a suggested fix.
    ///
    /// # Returns
    ///
    /// Returns `true` if the issue was reported; `false` if it was suppressed.
    #[inline]
    pub fn propose<F>(&mut self, issue: Issue, f: F) -> bool
    where
        F: FnOnce(&mut FixPlan),
    {
        let mut plan = FixPlan::new();
        f(&mut plan);

        let issue = issue.with_suggestion(self.semantics.source.identifier, plan);

        self.report(issue)
    }

    /// Recursively lints an AST node.
    ///
    /// This method applies the current rule's `lint_node` method to the node and recurses into its children
    /// based on the directive returned:
    /// - **Continue:** Lint the children.
    /// - **Prune:** Skip the children.
    /// - **Abort:** Abort the current branch.
    ///
    /// After processing the node, `filter_unused()` is called to move any ignore directives that apply to the
    /// node into the `unused_ignores` vector.
    ///
    /// # Parameters
    ///
    /// - `ast_node`: The current AST node being linted.
    ///
    /// # Returns
    ///
    /// Returns `true` if the linting process should be aborted for the current branch; otherwise, `false`.
    #[inline]
    pub(crate) fn lint(&mut self, ast_node: &AstNode<'_>) -> bool {
        if self.ignores(ast_node.node) {
            return false;
        }

        // Apply the lint rule to the current node.
        let directive = self.rule.rule.lint_node(ast_node.node, self);

        match directive {
            LintDirective::Continue => {
                // Recurse into each child node.
                for child in &ast_node.children {
                    if self.lint(child) {
                        return true;
                    }
                }

                false
            }
            LintDirective::Prune => false, // Skip children.
            LintDirective::Abort => true,  // Abort the current branch.
        }
    }

    /// Finalizes the linting context by reporting any remaining unused ignore directives.
    ///
    /// This method drains any ignore directives still left in the active pool (adding them to `unused_ignores`)
    /// and then reports each unused ignore as an issue. Each reported issue includes a primary annotation
    /// indicating the ignore's source span, a note explaining that the directive did not match any node in the AST,
    /// and a help message suggesting that it be removed or updated.
    ///
    /// # Returns
    ///
    /// An iterator over all reported issues.
    #[inline]
    pub fn finish(mut self) -> impl Iterator<Item = Issue> {
        // Move any remaining active ignores into unused_ignores.
        for ignore in self.ignores.drain(..) {
            self.unused_ignores.push(ignore);
        }

        // Report each unused ignore as an issue.
        let mut issues = std::mem::take(&mut self.issues);
        issues.extend(self.unused_ignores.drain(..).map(|ignore| {
            Issue::help("Unused ignore directive")
                .with_code(&self.rule.slug)
                .with_annotation(
                    Annotation::primary(ignore.span)
                        .with_message("This ignore directive was not used and may be removed."),
                )
                .with_note("It appears that this ignore directive does not match any node in the AST.")
                .with_help("Consider removing or updating this directive.")
        }));

        issues.into_iter()
    }
}
