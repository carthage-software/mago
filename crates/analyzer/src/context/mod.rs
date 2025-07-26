#![allow(dead_code)]

use itertools::Itertools;

use mago_codex::data_flow::graph::GraphKind;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::ReferenceSource;
use mago_codex::ttype::resolution::TypeResolutionContext;
use mago_docblock::document::Document;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;
use mago_names::scope::NamespaceScope;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_source::Source;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Trivia;
use mago_syntax::comments;

use crate::analysis_result::AnalysisResult;
use crate::artifacts::AnalysisArtifacts;
use crate::context::assertion::AssertionContext;
use crate::context::block::BlockContext;
use crate::context::scope::loop_scope::LoopScope;
use crate::issue::TypingIssueBuffer;
use crate::issue::TypingIssueKind;
use crate::settings::Settings;

pub mod assertion;
pub mod block;
pub mod scope;

#[derive(Debug)]
pub struct Context<'a> {
    pub(super) interner: &'a ThreadedInterner,
    pub(super) codebase: &'a CodebaseMetadata,
    pub(super) source: &'a Source,
    pub(super) resolved_names: &'a ResolvedNames,
    pub(super) loop_scope: Option<LoopScope>,
    pub(super) type_resolution_context: TypeResolutionContext,
    pub(super) comments: &'a [Trivia],
    pub(super) settings: &'a Settings,
    pub(super) scope: NamespaceScope,
    pub(super) buffer: TypingIssueBuffer,
    /// The span of the statement being analyzed.
    pub(super) statement_span: Span,
}

impl<'a> Context<'a> {
    pub fn new(
        interner: &'a ThreadedInterner,
        codebase: &'a CodebaseMetadata,
        source: &'a Source,
        resolved_names: &'a ResolvedNames,
        settings: &'a Settings,
        statement_span: Span,
        comments: &'a [Trivia],
    ) -> Self {
        Self {
            interner,
            codebase,
            source,
            resolved_names,
            loop_scope: None,
            type_resolution_context: TypeResolutionContext::new(),
            comments,
            settings,
            scope: NamespaceScope::default(),
            buffer: TypingIssueBuffer::new(),
            statement_span,
        }
    }

    pub fn set_loop_scope(&mut self, loop_scope: LoopScope) {
        let previous_scope = self.loop_scope.take().map(Box::new);
        self.loop_scope = Some(loop_scope.with_parent_loop(previous_scope));
    }

    pub fn take_loop_scope(&mut self) -> Option<LoopScope> {
        let mut loop_scope = self.loop_scope.take()?;
        match loop_scope.parent_loop.take() {
            Some(parent_loop) => {
                self.loop_scope = Some(*parent_loop);
            }
            None => {
                self.loop_scope = None;
            }
        }

        Some(loop_scope)
    }

    pub fn get_loop_scope(&self) -> Option<&LoopScope> {
        self.loop_scope.as_ref()
    }

    pub fn get_loop_scope_mut(&mut self) -> Option<&mut LoopScope> {
        self.loop_scope.as_mut()
    }

    pub fn get_assertion_context_from_block<'b>(&'b self, block_context: &'a BlockContext<'_>) -> AssertionContext<'b> {
        self.get_assertion_context(
            block_context.scope.get_class_like_name(),
            block_context.scope.get_reference_source(&self.source.identifier),
        )
    }

    #[inline]
    pub fn get_assertion_context<'b>(
        &'b self,
        this_class_name: Option<&'a StringIdentifier>,
        reference_source: Option<ReferenceSource>,
    ) -> AssertionContext<'b> {
        AssertionContext {
            file_source: self.source,
            resolved_names: self.resolved_names,
            interner: self.interner,
            codebase: self.codebase,
            this_class_name,
            type_resolution_context: &self.type_resolution_context,
            reference_source,
            settings: self.settings,
            in_loop: self.loop_scope.is_some(),
        }
    }

    pub fn get_docblock(&self) -> Option<&'a Trivia> {
        comments::docblock::get_docblock_before_position(
            self.interner,
            self.source,
            self.comments,
            self.statement_span.start.offset,
        )
    }

    pub fn get_parsed_docblock(&mut self) -> Option<Document> {
        let trivia = self.get_docblock()?;

        match mago_docblock::parse_trivia(self.interner, trivia) {
            Ok(document) => Some(document),
            Err(error) => {
                let error_span = error.span();

                let mut issue = Issue::error(error.to_string())
                    .with_annotation(
                        Annotation::primary(error_span).with_message("This part of the docblock has a syntax error"),
                    )
                    .with_note(error.note());

                if trivia.span != error_span {
                    issue = issue.with_annotation(
                        Annotation::secondary(trivia.span).with_message("The error is within this docblock"),
                    );
                }

                issue = issue.with_annotation(
                    Annotation::secondary(self.statement_span)
                        .with_message("This docblock is associated with the following statement"),
                );

                issue = issue.with_help(error.help());

                self.buffer.report(TypingIssueKind::InvalidDocblock, issue);

                None
            }
        }
    }

    pub fn record<T>(&mut self, callback: impl FnOnce(&mut Context<'a>) -> T) -> (T, Vec<Issue>) {
        let issue_counts = std::mem::take(&mut self.buffer.issue_counts);
        let mut issues = std::mem::take(&mut self.buffer.issues);

        let result = callback(self);

        self.buffer.issue_counts = issue_counts;
        std::mem::swap(&mut self.buffer.issues, &mut issues);

        (result, issues)
    }

    pub fn finish(self, artifacts: AnalysisArtifacts, analysis_result: &mut AnalysisResult, ignore_taint_path: bool) {
        analysis_result
            .emitted_issues
            .entry(self.source.identifier)
            .or_default()
            .extend(self.buffer.issues.into_iter().unique().collect::<Vec<_>>());

        if let GraphKind::WholeProgram = &artifacts.data_flow_graph.kind {
            if !ignore_taint_path {
                analysis_result.program_dataflow_graph.add_graph(artifacts.data_flow_graph);
            }
        } else {
            analysis_result.symbol_references.extend(artifacts.symbol_references);

            for (source_id, c) in artifacts.data_flow_graph.mixed_source_counts {
                if let Some(existing_count) = analysis_result.mixed_source_counts.get_mut(&source_id) {
                    existing_count.extend(c);
                } else {
                    analysis_result.mixed_source_counts.insert(source_id, c);
                }
            }

            for (kind, count) in self.buffer.issue_counts {
                *analysis_result.issue_counts.entry(kind.to_string()).or_insert(0) += count;
            }
        }
    }
}
