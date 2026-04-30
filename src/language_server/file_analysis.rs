//! Per-file derived data computed in a single parse + name-resolve pass.
//!
//! Each [`FileAnalysis`] owns its own [`Bump`] arena. The arena keeps
//! the parse tree and [`ResolvedNames`] alive for the lifetime of the
//! analysis, so capability handlers can use `mago_names`'s resolution
//! map directly instead of going through an owned shadow copy.

use std::sync::Arc;

use bumpalo::Bump;

use mago_database::file::File as MagoFile;
use mago_linter::Linter;
use mago_names::ResolvedNames;
use mago_names::resolver::NameResolver;
use mago_reporting::IssueCollection;
use mago_semantics::SemanticsChecker;
use mago_span::HasSpan;
use mago_syntax::ast::Block;
use mago_syntax::ast::Class;
use mago_syntax::ast::Closure;
use mago_syntax::ast::Enum;
use mago_syntax::ast::Function;
use mago_syntax::ast::If;
use mago_syntax::ast::Interface;
use mago_syntax::ast::Match;
use mago_syntax::ast::Method;
use mago_syntax::ast::Namespace;
use mago_syntax::ast::Switch;
use mago_syntax::ast::Trait;
use mago_syntax::ast::Try;
use mago_syntax::parser::parse_file_with_settings;
use mago_syntax::walker::Walker;
use mago_syntax::walker::walk_program;
use tower_lsp::lsp_types::FoldingRange;
use tower_lsp::lsp_types::FoldingRangeKind;

use crate::language_server::linter::LinterContext;

/// Owned, cacheable view of one file. Built by [`build`]; held on the
/// workspace state for every file that's ever been touched.
pub struct FileAnalysis {
    pub lint_issues: IssueCollection,
    pub fold_ranges: Vec<FoldingRange>,
    /// AST node spans (block-like constructs) sorted by start. Used by
    /// `selection_range` to answer "what spans contain this offset?"
    /// without re-walking the AST.
    pub node_spans: Vec<(u32, u32)>,
    /// Self-referential: `&'static str` values inside borrow from
    /// `_arena`. Accessed only via [`Self::resolved`], which downcasts
    /// to a `'self`-bound reference.
    resolved: ResolvedNames<'static>,
    /// Boxed so the `Bump`'s address is stable across moves of
    /// `FileAnalysis`. Treated as frozen storage after construction:
    /// nothing reaches it, nothing allocates, nothing resets. When the
    /// analysis is dropped, the heap chunks are freed and the
    /// references in `resolved` are invalidated at the same instant.
    _arena: Box<Bump>,
}

// SAFETY: After construction the `Bump` is never accessed via a shared
// reference: no `alloc` (which mutates allocator state through `&self`)
// and no `reset` (which requires `&mut self`) is reachable. The `&str`
// values in `resolved` point at immutable bytes in heap chunks the
// `Bump` owns; reading immutable bytes from multiple threads is safe.
unsafe impl Sync for FileAnalysis {}

impl std::fmt::Debug for FileAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileAnalysis")
            .field("lint_issues", &self.lint_issues.len())
            .field("fold_ranges", &self.fold_ranges.len())
            .field("node_spans", &self.node_spans.len())
            .field("resolved_names", &self.resolved.len())
            .finish()
    }
}

impl FileAnalysis {
    /// Borrow the resolved-name map. The lifetime is rebound to `&self`
    /// (covariance on `ResolvedNames<'a>`'s lifetime parameter), so the
    /// returned reference can't outlive the owning analysis.
    pub fn resolved(&self) -> &ResolvedNames<'_> {
        &self.resolved
    }
}

/// Run one parse + resolve pass over `file` and extract every per-file
/// derivative the LSP needs.
///
/// `with_semantics = true` runs the [`SemanticsChecker`] alongside the
/// rule pass; set it only when the analyzer isn't running on the same
/// file (otherwise semantic issues get reported twice).
pub fn build(file: &MagoFile, linter_ctx: &LinterContext, with_semantics: bool) -> FileAnalysis {
    let arena: Box<Bump> = Box::new(Bump::new());

    // SAFETY: `arena` is moved into the returned `FileAnalysis` at the
    // end of this function. Until then it stays at a stable heap address
    // (it's already boxed, and Box doesn't move its allocation on move).
    // Borrows derived from `arena_ref` end up stored in the same struct
    // as `arena` itself, so they're freed together.
    let arena_ref: &'static Bump = unsafe { &*(arena.as_ref() as *const Bump) };

    let program = parse_file_with_settings(arena_ref, file, linter_ctx.parser_settings);
    let resolved = NameResolver::new(arena_ref).resolve(program);

    let mut lint_issues = IssueCollection::new();
    if with_semantics {
        let checker = SemanticsChecker::new(linter_ctx.settings.php_version);
        lint_issues.extend(checker.check(file, program, &resolved));
    }
    let linter = Linter::from_registry(arena_ref, Arc::clone(&linter_ctx.registry), linter_ctx.settings.php_version);
    lint_issues.extend(linter.lint(file, program, &resolved));

    let mut span_ctx = SpanCollectCtx { fold_ranges: Vec::new(), node_spans: Vec::new(), file };
    walk_program(&SpanCollector, program, &mut span_ctx);
    push_comment_ranges(file, &mut span_ctx.fold_ranges);
    span_ctx.node_spans.sort_unstable();
    span_ctx.node_spans.dedup();

    FileAnalysis {
        lint_issues,
        fold_ranges: span_ctx.fold_ranges,
        node_spans: span_ctx.node_spans,
        resolved,
        _arena: arena,
    }
}

struct SpanCollectCtx<'a> {
    fold_ranges: Vec<FoldingRange>,
    node_spans: Vec<(u32, u32)>,
    file: &'a MagoFile,
}

impl SpanCollectCtx<'_> {
    fn record_block_like(&mut self, start: u32, end: u32) {
        self.node_spans.push((start, end));
        let start_line = self.file.line_number(start);
        let end_line = self.file.line_number(end);
        if end_line > start_line {
            self.fold_ranges.push(FoldingRange {
                start_line,
                start_character: None,
                end_line,
                end_character: None,
                kind: None,
                collapsed_text: None,
            });
        }
    }

    fn record_node(&mut self, start: u32, end: u32) {
        self.node_spans.push((start, end));
    }
}

struct SpanCollector;

impl<'arena> Walker<'arena, 'arena, SpanCollectCtx<'_>> for SpanCollector {
    fn walk_in_block(&self, n: &'arena Block<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_block_like(n.span().start.offset, n.span().end.offset);
    }
    fn walk_in_class(&self, n: &'arena Class<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_block_like(n.span().start.offset, n.span().end.offset);
    }
    fn walk_in_interface(&self, n: &'arena Interface<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_block_like(n.span().start.offset, n.span().end.offset);
    }
    fn walk_in_trait(&self, n: &'arena Trait<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_block_like(n.span().start.offset, n.span().end.offset);
    }
    fn walk_in_enum(&self, n: &'arena Enum<'arena>, c: &mut SpanCollectCtx<'_>) {
        c.record_block_like(n.span().start.offset, n.span().end.offset);
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

fn push_comment_ranges(file: &MagoFile, out: &mut Vec<FoldingRange>) {
    let text = file.contents.as_ref();
    let mut search_start = 0;
    while let Some(rel_open) = text[search_start..].find("/*") {
        let open = search_start + rel_open;
        let after = open + 2;
        if let Some(rel_close) = text[after..].find("*/") {
            let close_end = after + rel_close + 2;
            let start_line = file.line_number(open as u32);
            let end_line = file.line_number(close_end as u32);
            if end_line > start_line {
                out.push(FoldingRange {
                    start_line,
                    start_character: None,
                    end_line,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Comment),
                    collapsed_text: None,
                });
            }
            search_start = close_end;
        } else {
            break;
        }
    }
}
