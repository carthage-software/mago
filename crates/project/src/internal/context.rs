use mago_ast::Program;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_names::Names;
use mago_php_version::PHPVersion;
use mago_reporting::IssueCollection;
use mago_source::Source;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

#[derive(Debug)]
pub struct Context<'i, 'alloc> {
    pub interner: &'i ThreadedInterner,
    pub version: PHPVersion,
    pub program: &'alloc Program<'alloc>,
    pub names: &'i Names,
    pub source: &'i Source,
    pub issues: IssueCollection,
    pub ancestors: Vec<Span>,
    pub hint_depth: usize,
}

impl<'i, 'alloc> Context<'i, 'alloc> {
    pub fn new(
        interner: &'i ThreadedInterner,
        version: PHPVersion,
        program: &'alloc Program<'alloc>,
        names: &'i Names,
        source: &'i Source,
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
    pub fn get_name(&self, position: &Position) -> &'i str {
        self.interner.lookup(self.names.get(position))
    }

    #[inline]
    pub fn get_code_snippet(&self, span: impl HasSpan) -> &'i str {
        fn get_code_snippet_of_span<'i>(i: &'i ThreadedInterner, c: &StringIdentifier, s: &Span) -> &'i str {
            let source = i.lookup(c);

            &source[s.start.offset..s.end.offset]
        }

        get_code_snippet_of_span(self.interner, &self.source.content, &span.span())
    }

    #[inline]
    pub fn take_issue_collection(self) -> IssueCollection {
        self.issues
    }
}
