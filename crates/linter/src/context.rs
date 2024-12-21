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

    /// Retrieves the name associated with a given position in the code.
    ///
    /// # Panics
    ///
    /// Panics if no name is found at the specified position.
    pub fn lookup_name(&self, position: &impl HasPosition) -> &str {
        let name_id = self.semantics.names.get(&position.position());

        self.lookup(name_id)
    }

    pub fn lookup_function_name(&self, identifier: &Identifier) -> &str {
        if self.is_name_imported(identifier) {
            self.lookup_name(identifier)
        } else {
            let name = self.lookup(&identifier.value());

            if let Some(stripped) = name.strip_prefix('\\') {
                stripped
            } else {
                name
            }
        }
    }

    /// Checks if a name at a given position is imported.
    pub fn is_name_imported(&self, position: &impl HasPosition) -> bool {
        self.semantics.names.is_imported(&position.position())
    }

    /// Converts a type hint into a human-readable string representation.
    pub fn lookup_hint(&self, hint: &Hint) -> String {
        match hint {
            Hint::Identifier(identifier) => self.lookup_name(identifier).to_string(),
            Hint::Parenthesized(parenthesized_hint) => {
                format!("({})", self.lookup_hint(&parenthesized_hint.hint))
            }
            Hint::Nullable(nullable_hint) => format!("?{}", self.lookup_hint(&nullable_hint.hint)),
            Hint::Union(union_hint) => {
                format!("{}|{}", self.lookup_hint(&union_hint.left), self.lookup_hint(&union_hint.right))
            }
            Hint::Intersection(intersection_hint) => {
                format!("{}&{}", self.lookup_hint(&intersection_hint.left), self.lookup_hint(&intersection_hint.right))
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
