use bumpalo::Bump;
use mago_atom::Atom;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::resolution::TypeResolutionContext;
use mago_collector::Collector;
use mago_database::file::File;
use mago_docblock::document::Document;
use mago_names::ResolvedNames;
use mago_names::scope::NamespaceScope;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::Trivia;
use mago_syntax::comments;

use crate::analysis_result::AnalysisResult;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::assertion::AssertionContext;
use crate::context::block::BlockContext;
use crate::settings::Settings;

pub mod assertion;
pub mod block;
pub mod scope;
pub mod utils;

#[derive(Debug)]
pub struct Context<'ctx, 'arena> {
    pub(super) arena: &'arena Bump,
    pub(super) codebase: &'ctx CodebaseMetadata,
    pub(super) source_file: &'ctx File,
    pub(super) resolved_names: &'ctx ResolvedNames<'arena>,
    pub(super) type_resolution_context: TypeResolutionContext,
    pub(super) comments: &'arena [Trivia<'arena>],
    pub(super) settings: &'ctx Settings,
    pub(super) scope: NamespaceScope,
    pub(super) collector: Collector<'ctx, 'arena>,
    pub(super) statement_span: Span,
}

impl<'ctx, 'arena> Context<'ctx, 'arena> {
    pub fn new(
        arena: &'arena Bump,
        codebase: &'ctx CodebaseMetadata,
        source: &'ctx File,
        resolved_names: &'ctx ResolvedNames<'arena>,
        settings: &'ctx Settings,
        statement_span: Span,
        comments: &'arena [Trivia<'arena>],
        collector: Collector<'ctx, 'arena>,
    ) -> Self {
        Self {
            arena,
            codebase,
            source_file: source,
            resolved_names,
            type_resolution_context: TypeResolutionContext::new(),
            comments,
            settings,
            scope: NamespaceScope::default(),
            statement_span,
            collector,
        }
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
    /// within the codebase metadata.
    pub fn resolve_function_name<'ast>(&self, identifier: &'ast Identifier<'arena>) -> &'arena str {
        if self.resolved_names.is_imported(identifier) {
            return self.resolved_names.get(identifier);
        }

        let name = identifier.value();

        if let Some(stripped) = name.strip_prefix('\\') {
            return stripped;
        }

        let fqfn = self.resolved_names.get(&identifier);
        if self.codebase.function_exists(fqfn) {
            return fqfn;
        }

        if !name.contains('\\') && self.codebase.function_exists(name) {
            return name;
        }

        fqfn
    }

    pub fn get_assertion_context_from_block(
        &self,
        block_context: &BlockContext<'ctx>,
    ) -> AssertionContext<'ctx, 'arena> {
        self.get_assertion_context(block_context.scope.get_class_like_name())
    }

    #[inline]
    pub fn get_assertion_context(&self, this_class_name: Option<Atom>) -> AssertionContext<'ctx, 'arena> {
        AssertionContext {
            arena: self.arena,
            resolved_names: self.resolved_names,
            codebase: self.codebase,
            this_class_name,
        }
    }

    pub fn get_docblock(&self) -> Option<&'arena Trivia<'arena>> {
        comments::docblock::get_docblock_before_position(
            self.source_file,
            self.comments,
            self.statement_span.start.offset,
        )
    }

    pub fn get_parsed_docblock(&mut self) -> Option<Document<'arena>> {
        let trivia = self.get_docblock()?;

        match mago_docblock::parse_trivia(self.arena, trivia) {
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

                self.collector.report_with_code(IssueCode::InvalidDocblock, issue);

                None
            }
        }
    }

    pub fn record<T>(&mut self, callback: impl FnOnce(&mut Context<'ctx, 'arena>) -> T) -> (T, IssueCollection) {
        self.collector.start_recording();
        let result = callback(self);
        let issues = self.collector.finish_recording().unwrap_or_default();

        (result, issues)
    }

    pub fn finish(self, artifacts: AnalysisArtifacts, analysis_result: &mut AnalysisResult) {
        analysis_result.issues.extend(self.collector.finish());
        analysis_result.symbol_references.extend(artifacts.symbol_references);
    }
}
