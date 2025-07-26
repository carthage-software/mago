use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_source::Source;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;
use mago_syntax::ast::Program;

const ISSUE_CODE: &str = "semantics";

#[derive(Debug)]
pub struct Context<'a> {
    pub interner: &'a ThreadedInterner,
    pub version: &'a PHPVersion,
    pub program: &'a Program,
    pub names: &'a ResolvedNames,
    pub source: &'a Source,
    pub ancestors: Vec<Span>,
    pub hint_depth: usize,

    issues: IssueCollection,
}

impl<'a> Context<'a> {
    pub fn new(
        interner: &'a ThreadedInterner,
        version: &'a PHPVersion,
        program: &'a Program,
        names: &'a ResolvedNames,
        source: &'a Source,
    ) -> Self {
        Self {
            interner,
            version,
            program,
            names,
            source,
            issues: IssueCollection::default(),
            ancestors: vec![],
            hint_depth: 0,
        }
    }

    #[inline]
    pub fn get_name(&self, position: &Position) -> &'a str {
        self.interner.lookup(self.names.get(position))
    }

    #[inline]
    pub fn get_code_snippet(&self, span: impl HasSpan) -> &'a str {
        fn get_code_snippet_of_span<'a>(i: &'a ThreadedInterner, c: &StringIdentifier, s: &Span) -> &'a str {
            let source = i.lookup(c);

            &source[s.start.offset..s.end.offset]
        }

        get_code_snippet_of_span(self.interner, &self.source.content, &span.span())
    }

    /// Reports a semantic issue with the given `Issue`.
    ///
    /// This method adds the issue to the context's issue collection,
    /// appending the `ISSUE_CODE` to the issue for identification.
    ///
    /// # Arguments
    ///
    /// `issue`: The `Issue` to report, which contains details about the semantic violation.
    pub fn report(&mut self, issue: Issue) {
        self.issues.push(issue.with_code(ISSUE_CODE));
    }

    /// Finalizes the context and returns the collected issues.
    ///
    /// This method is typically called at the end of the semantic analysis
    /// to retrieve all reported issues.
    pub fn finalize(self) -> IssueCollection {
        self.issues
    }
}
