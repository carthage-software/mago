use fennec_ast::Identifier;
use fennec_semantics::Semantics;
use toml::value::Value;

use fennec_ast::Hint;
use fennec_fixer::FixPlan;
use fennec_interner::StringIdentifier;
use fennec_interner::ThreadedInterner;
use fennec_reporting::Issue;
use fennec_reporting::IssueCollection;
use fennec_reporting::Level;
use fennec_span::HasPosition;
use fennec_span::HasSpan;
use fennec_span::Span;

use crate::consts::ANONYMOUS_CLASS_NAME;
use crate::rule::ConfiguredRule;

#[derive(Debug)]
pub struct Context<'a> {
    pub interner: &'a ThreadedInterner,
    pub semantics: &'a Semantics,
    pub issues: IssueCollection,
}

impl<'a> Context<'a> {
    pub fn new(interner: &'a ThreadedInterner, semantics: &'a Semantics) -> Self {
        Self { interner, semantics, issues: IssueCollection::default() }
    }

    pub fn for_rule<'b>(&'b mut self, rule: &'b ConfiguredRule) -> LintContext<'b> {
        LintContext { rule: &rule, interner: &self.interner, semantics: &self.semantics, issues: &mut self.issues }
    }

    pub fn take_issue_collection(self) -> IssueCollection {
        self.issues
    }
}

#[derive(Debug)]
pub struct LintContext<'a> {
    pub rule: &'a ConfiguredRule,
    pub interner: &'a ThreadedInterner,
    pub semantics: &'a Semantics,
    pub issues: &'a mut IssueCollection,
}

impl<'a> LintContext<'a> {
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

        self.lookup(&name_id)
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

    pub fn get_class_like_details(&self, node: &impl HasSpan) -> (String, String, String, Span) {
        let class_like_symbol = self
            .semantics
            .symbols
            .get_enclosing_class_like(node.span().start.offset)
            .expect("expected to find a class-like symbol at the given position");

        let class_like_kind = class_like_symbol.kind.to_string();
        let class_like_span = class_like_symbol.span;
        let (class_like_name, class_like_fqcn) = class_like_symbol
            .identifier
            .map(|i| (self.lookup(&i.name).to_string(), self.lookup(&i.fully_qualified_name).to_string()))
            .unwrap_or_else(|| (ANONYMOUS_CLASS_NAME.to_string(), ANONYMOUS_CLASS_NAME.to_string()));

        (class_like_kind, class_like_name, class_like_fqcn, class_like_span)
    }
}
