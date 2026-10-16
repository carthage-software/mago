//! Per-file derived data computed in a single parse + name-resolve pass.
//!
//! Each [`FileAnalysis`] keeps owned names and spans. The build drops its
//! parse tree and arena before returning.

use std::sync::Arc;

use mago_allocator::LocalArena;
use mago_database::file::File as MagoFile;
use mago_linter::Linter;
use mago_names::resolver::NameResolver;
use mago_names::scope::NamespaceScope;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_semantics::SemanticsChecker;
use mago_span::HasSpan;
use mago_syntax::cst::AttributeList;
use mago_syntax::cst::Block;
use mago_syntax::cst::Class;
use mago_syntax::cst::Closure;
use mago_syntax::cst::Enum;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Function;
use mago_syntax::cst::If;
use mago_syntax::cst::Interface;
use mago_syntax::cst::Match;
use mago_syntax::cst::Method;
use mago_syntax::cst::Namespace;
use mago_syntax::cst::Program;
use mago_syntax::cst::Statement;
use mago_syntax::cst::Switch;
use mago_syntax::cst::Trait;
use mago_syntax::cst::Try;
use mago_syntax::parser::parse_file_borrowed_with_settings;
use mago_syntax::walker::Walker;
use mago_syntax::walker::walk_program;

use crate::Range;
use crate::call_analysis::CallSite;
use crate::domain::DocumentSymbolItem;
use crate::domain::FoldKind;
use crate::domain::FoldRange;
use crate::linter::LinterContext;
use crate::resolved_names::ResolvedNames;
use crate::semantic_index::SemanticIndex;

pub(crate) mod cache;

/// Owned, cacheable view of one file. Built by [`build`]; held on the
/// workspace state for every file that's ever been touched.
pub struct FileAnalysis {
    pub document_symbols: Vec<DocumentSymbolItem>,
    pub lint_issues: IssueCollection,
    pub fold_ranges: Vec<FoldRange>,
    /// Statement and block-like node spans sorted by start. Used by
    /// `selection_range` to answer "what spans contain this offset?"
    /// without re-walking the AST.
    pub node_spans: Vec<(u32, u32)>,
    /// Expression spans, including scalars, used to select one inferred type.
    pub expression_spans: Box<[(u32, u32)]>,
    /// Actual calls; creating a first-class callable does not invoke it.
    pub(crate) calls: Box<[CallSite]>,
    /// Owned names and source spans, sorted by start offset.
    resolved: ResolvedNames,
    semantic: SemanticIndex,
    /// Per-namespace import scopes, sorted by start offset, each covering
    /// `[start, end)`. Owned (no arena borrow), so completion can resolve names
    /// exactly as the resolver did without re-parsing `use` statements.
    scopes: Vec<(u32, u32, NamespaceScope)>,
}

impl std::fmt::Debug for FileAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileAnalysis")
            .field("lint_issues", &self.lint_issues.len())
            .field("fold_ranges", &self.fold_ranges.len())
            .field("node_spans", &self.node_spans.len())
            .field("resolved_names", &self.resolved.len())
            .finish_non_exhaustive()
    }
}

impl FileAnalysis {
    /// Borrow the resolved names for this file.
    #[must_use]
    pub fn resolved(&self) -> &ResolvedNames {
        &self.resolved
    }

    #[must_use]
    pub fn semantic(&self) -> &SemanticIndex {
        &self.semantic
    }

    /// The import scope in effect at `offset` (aliases plus the current
    /// namespace), or the global scope when the offset lies outside every
    /// namespace declaration.
    #[must_use]
    pub fn scope_at(&self, offset: u32) -> NamespaceScope {
        self.scopes
            .iter()
            .find(|(start, end, _)| *start <= offset && offset < *end)
            .map(|(_, _, scope)| scope.clone())
            .unwrap_or_default()
    }
}

/// Build the import scope for every namespace in the program (and a global
/// scope for code outside any namespace), reusing the name resolver's own
/// [`NamespaceScope::populate_from_use`]. Each entry covers `[start, end)`.
fn collect_scopes(program: &Program<'_>, file_size: u32) -> Vec<(u32, u32, NamespaceScope)> {
    let mut out = Vec::new();
    let mut global = NamespaceScope::global();
    let mut saw_namespace = false;

    for statement in program.statements.iter() {
        match statement {
            Statement::Namespace(namespace) => {
                saw_namespace = true;
                let mut scope = match &namespace.name {
                    Some(name) => NamespaceScope::for_namespace(name.value().to_vec()),
                    None => NamespaceScope::global(),
                };
                for inner in namespace.statements().iter() {
                    if let Statement::Use(r#use) = inner {
                        scope.populate_from_use(r#use);
                    }
                }
                let span = namespace.span();
                out.push((span.start.offset, span.end.offset, scope));
            }
            Statement::Use(r#use) => global.populate_from_use(r#use),
            _ => {}
        }
    }

    if !saw_namespace {
        out.push((0, file_size, global));
    }

    out.sort_by_key(|(start, _, _)| *start);
    out
}

/// Run one parse + resolve pass over `file` and extract every per-file
/// derivative the LSP needs. Only host files produce diagnostics; dependency
/// queries still build the data needed for navigation.
///
/// `with_semantics = true` runs the [`SemanticsChecker`] alongside the
/// rule pass; set it only when the analyzer isn't running on the same
/// file (otherwise semantic issues get reported twice).
#[must_use]
pub fn build(file: &MagoFile, linter_ctx: &LinterContext, with_semantics: bool) -> FileAnalysis {
    let arena = LocalArena::new();
    let program = parse_file_borrowed_with_settings(&arena, file, linter_ctx.parser_settings);
    let resolved = NameResolver::new(&arena).resolve(program);
    let scopes = collect_scopes(program, file.size);

    let mut lint_issues = IssueCollection::new();
    if file.file_type.is_host() && with_semantics {
        lint_issues.extend(program.errors.iter().map(Issue::from));
        let checker = SemanticsChecker::new(linter_ctx.settings.php_version);
        lint_issues.extend(checker.check(file, program, &resolved));
    }
    if let Some(registry) = linter_ctx.registry_for(file) {
        let linter = Linter::from_registry(&arena, Arc::clone(registry), linter_ctx.settings.php_version);
        lint_issues.extend(linter.lint(file, program, &resolved));
    }

    let mut span_ctx = SpanCollectCtx {
        fold_ranges: Vec::new(),
        node_ranges: Vec::new(),
        expression_ranges: Vec::new(),
        calls: Vec::new(),
        file,
    };
    walk_program(&SpanCollector, program, &mut span_ctx);
    push_comment_ranges(file, program, &mut span_ctx.fold_ranges);
    span_ctx.node_ranges.sort_unstable();
    span_ctx.node_ranges.dedup();
    span_ctx.expression_ranges.sort_unstable();
    span_ctx.expression_ranges.dedup();
    span_ctx.calls.sort_unstable_by_key(|call| call.selection.start);

    FileAnalysis {
        document_symbols: crate::capabilities::document_symbol::collect(program),
        lint_issues,
        fold_ranges: span_ctx.fold_ranges,
        node_spans: span_ctx.node_ranges,
        expression_spans: span_ctx.expression_ranges.into_boxed_slice(),
        calls: span_ctx.calls.into_boxed_slice(),
        resolved: ResolvedNames::from(&resolved),
        semantic: crate::semantic_index::build(program, &resolved, file.contents.as_ref()),
        scopes,
    }
}

struct SpanCollectCtx<'file> {
    fold_ranges: Vec<FoldRange>,
    node_ranges: Vec<(u32, u32)>,
    expression_ranges: Vec<(u32, u32)>,
    calls: Vec<CallSite>,
    file: &'file MagoFile,
}

impl SpanCollectCtx<'_> {
    fn record_block_like(&mut self, start: u32, end: u32) {
        self.node_ranges.push((start, end));
        self.record_fold(start, end);
    }

    fn record_fold(&mut self, start: u32, end: u32) {
        let start_line = self.file.line_number(start);
        let end_line = self.file.line_number(end);
        if end_line > start_line {
            self.fold_ranges.push(FoldRange { range: Range::new(start, end), kind: None });
        }
    }

    fn record_node(&mut self, start: u32, end: u32) {
        self.node_ranges.push((start, end));
    }
}

struct SpanCollector;

impl<'arena> Walker<'arena, 'arena, SpanCollectCtx<'_>> for SpanCollector {
    fn walk_in_statement(&self, n: &'arena Statement<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_node(n.start_offset(), n.end_offset());
    }

    fn walk_in_expression(&self, n: &'arena Expression<'arena>, c: &mut SpanCollectCtx<'_>) {
        if let Some(call) = CallSite::from_expression(n) {
            c.calls.push(call);
        }
        // A call's identifier selects the call result; class names use resolved names.
        if !matches!(n, Expression::Identifier(_)) {
            c.expression_ranges.push((n.start_offset(), n.end_offset()));
        }
    }

    fn walk_in_block(&self, n: &'arena Block<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_block_like(n.span().start.offset, n.span().end.offset);
    }

    fn walk_in_class(&self, n: &'arena Class<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_node(n.start_offset(), n.end_offset());
        c.record_fold(n.left_brace.start.offset, n.right_brace.end.offset);
    }

    fn walk_in_interface(&self, n: &'arena Interface<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_node(n.start_offset(), n.end_offset());
        c.record_fold(n.left_brace.start.offset, n.right_brace.end.offset);
    }

    fn walk_in_trait(&self, n: &'arena Trait<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_node(n.start_offset(), n.end_offset());
        c.record_fold(n.left_brace.start.offset, n.right_brace.end.offset);
    }

    fn walk_in_enum(&self, n: &'arena Enum<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_node(n.start_offset(), n.end_offset());
        c.record_fold(n.left_brace.start.offset, n.right_brace.end.offset);
    }

    fn walk_in_attribute_list(&self, n: &'arena AttributeList<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_fold(n.start_offset(), n.end_offset());
    }

    fn walk_in_match(&self, n: &'arena Match<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_block_like(n.span().start.offset, n.span().end.offset);
    }

    fn walk_in_switch(&self, n: &'arena Switch<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_block_like(n.span().start.offset, n.span().end.offset);
    }

    fn walk_in_namespace(&self, n: &'arena Namespace<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_node(n.span().start.offset, n.span().end.offset);
    }

    fn walk_in_function(&self, n: &'arena Function<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_node(n.span().start.offset, n.span().end.offset);
    }

    fn walk_in_method(&self, n: &'arena Method<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_node(n.span().start.offset, n.span().end.offset);
    }

    fn walk_in_closure(&self, n: &'arena Closure<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_node(n.span().start.offset, n.span().end.offset);
    }

    fn walk_in_if(&self, n: &'arena If<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_node(n.span().start.offset, n.span().end.offset);
    }

    fn walk_in_try(&self, n: &'arena Try<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_node(n.span().start.offset, n.span().end.offset);
    }
}

fn push_comment_ranges(file: &MagoFile, program: &Program<'_>, out: &mut Vec<FoldRange>) {
    for trivia in program.trivia.iter().filter(|trivia| trivia.kind.is_block_comment()) {
        let start = trivia.span.start.offset;
        let end = trivia.span.end.offset;
        if file.line_number(end) > file.line_number(start) {
            out.push(FoldRange { range: Range::new(start, end), kind: Some(FoldKind::Comment) });
        }
    }
}
