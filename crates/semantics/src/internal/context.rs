use mago_database::file::File;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;
use mago_syntax::ast::Program;

const ISSUE_CODE: &str = "semantics";

#[derive(Debug)]
pub struct Context<'ctx, 'ast, 'arena> {
    pub version: PHPVersion,
    pub program: &'ast Program<'arena>,
    pub names: &'ast ResolvedNames<'arena>,
    pub source_file: &'ctx File,
    pub ancestors: Vec<Span>,
    pub hint_depth: usize,

    issues: IssueCollection,
}

impl<'ctx, 'ast, 'arena> Context<'ctx, 'ast, 'arena> {
    pub fn new(
        version: PHPVersion,
        program: &'ast Program<'arena>,
        names: &'ast ResolvedNames<'arena>,
        source_file: &'ctx File,
    ) -> Self {
        Self {
            version,
            program,
            names,
            source_file,
            issues: IssueCollection::default(),
            ancestors: vec![],
            hint_depth: 0,
        }
    }

    #[inline]
    pub fn get_name(&self, position: &Position) -> &'arena str {
        self.names.get(position)
    }

    #[inline]
    pub fn get_code_snippet(&self, span: impl HasSpan) -> &'ctx str {
        let s = span.span();

        &self.source_file.contents[s.start.offset as usize..s.end.offset as usize]
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
