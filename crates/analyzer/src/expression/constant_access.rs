use mago_codex::get_constant;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_mixed;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;

impl Analyzable for ConstantAccess {
    fn analyze(
        &self,
        context: &mut Context<'_>,
        _block_context: &mut BlockContext,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let name = context.resolved_names.get(self);
        let unqualified_name = self.name.value();

        let constant_metadata = get_constant(context.codebase, context.interner, name)
            .or_else(|| get_constant(context.codebase, context.interner, unqualified_name));

        let Some(constant_metadata) = constant_metadata else {
            let constant_name = context.interner.lookup(name);

            context.buffer.report(
                TypingIssueKind::NonExistentConstant,
                Issue::error(format!(
                    "Undefined constant: `{constant_name}`."
                ))
                .with_annotation(
                    Annotation::primary(self.span())
                        .with_message(format!("Constant `{constant_name}` is not defined."))
                )
                .with_note(
                    "The constant might be misspelled, not defined, or not imported."
                )
                .with_help(
                    format!(
                        "Define the constant `{constant_name}` using `define()` or `const`, or check for typos and ensure it's available in this scope."
                    )
                ),
            );

            return Ok(());
        };

        if constant_metadata.is_deprecated {
            let constant_name = context.interner.lookup(name);

            context.buffer.report(
                TypingIssueKind::DeprecatedConstant,
                Issue::warning(format!("Using deprecated constant: `{constant_name}`."))
                    .with_annotation(Annotation::primary(self.span()).with_message("This constant is deprecated."))
                    .with_note("Consider using an alternative constant or variable.")
                    .with_help("Check `{constant_name}` documentation for alternatives or updates."),
            );
        }

        let mut constant_type = constant_metadata.inferred_type.clone().unwrap_or_else(get_mixed);

        expander::expand_union(
            context.codebase,
            context.interner,
            &mut constant_type,
            &TypeExpansionOptions::default(),
        );

        artifacts.set_expression_type(self, constant_type);

        Ok(())
    }
}
