use toml::value::Value;

use mago_ast::Hint;
use mago_ast::Identifier;
use mago_fixer::FixPlan;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_reflection::CodebaseReflection;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_semantics::Semantics;
use mago_span::HasPosition;

use crate::rule::ConfiguredRule;

#[derive(Debug)]
pub struct Context<'a> {
    pub interner: &'a ThreadedInterner,
    pub codebase: &'a CodebaseReflection,
    pub semantics: &'a Semantics,
    pub issues: IssueCollection,
}

impl<'a> Context<'a> {
    pub fn new(interner: &'a ThreadedInterner, codebase: &'a CodebaseReflection, semantics: &'a Semantics) -> Self {
        Self { interner, codebase, semantics, issues: IssueCollection::default() }
    }

    pub fn for_rule<'b>(&'b mut self, rule: &'b ConfiguredRule) -> LintContext<'b> {
        LintContext {
            rule,
            interner: self.interner,
            codebase: self.codebase,
            semantics: self.semantics,
            issues: &mut self.issues,
        }
    }

    pub fn take_issue_collection(self) -> IssueCollection {
        self.issues
    }
}

#[derive(Debug)]
pub struct LintContext<'a> {
    pub rule: &'a ConfiguredRule,
    pub interner: &'a ThreadedInterner,
    pub codebase: &'a CodebaseReflection,
    pub semantics: &'a Semantics,
    pub issues: &'a mut IssueCollection,
}

impl LintContext<'_> {
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
        if self.codebase.function_exists(&self.interner.lowered(fqfn_id)) {
            // The FQFN exists, so return it.
            return self.lookup(fqfn_id);
        }

        // If FQFN doesn't exist, check if the global function name exists.
        if !name.contains('\\') && self.codebase.function_exists(&self.interner.lowered(&name_id)) {
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
        if self.codebase.constant_exists(fqcn_id) {
            // The FQCN exists, so return it.
            return self.lookup(fqcn_id);
        }

        // If FQCN doesn't exist, check if the global constant name exists.
        if !name.contains('\\') && self.codebase.constant_exists(&name_id) {
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

    pub fn report(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    pub fn report_with_fix<F>(&mut self, issue: Issue, f: F)
    where
        F: FnOnce(&mut FixPlan),
    {
        let mut plan = FixPlan::new();
        f(&mut plan);

        let issue = issue.with_suggestion(self.semantics.source.identifier, plan);

        self.report(issue);
    }
}
