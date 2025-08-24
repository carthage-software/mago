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
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ConstantAccess<'arena> {
    fn analyze<'ctx>(
        &self,
        context: &mut Context<'ctx, 'arena>,
        _block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let name = context.resolved_names.get(self);
        let unqualified_name = self.name.value();

        let constant_metadata =
            get_constant(context.codebase, name).or_else(|| get_constant(context.codebase, unqualified_name));

        let Some(constant_metadata) = constant_metadata else {
            context.collector.report_with_code(
                IssueCode::NonExistentConstant,
                Issue::error(format!(
                    "Undefined constant: `{name}`."
                ))
                .with_annotation(
                    Annotation::primary(self.span())
                        .with_message(format!("Constant `{name}` is not defined."))
                )
                .with_note(
                    "The constant might be misspelled, not defined, or not imported."
                )
                .with_help(
                    format!(
                        "Define the constant `{name}` using `define()` or `const`, or check for typos and ensure it's available in this scope."
                    )
                ),
            );

            return Ok(());
        };

        if constant_metadata.flags.is_deprecated() {
            context.collector.report_with_code(
                IssueCode::DeprecatedConstant,
                Issue::warning(format!("Using deprecated constant: `{name}`."))
                    .with_annotation(Annotation::primary(self.span()).with_message("This constant is deprecated."))
                    .with_note("Consider using an alternative constant or variable.")
                    .with_help("Check `{name}` documentation for alternatives or updates."),
            );
        }

        let mut constant_type = constant_metadata.inferred_type.clone().unwrap_or_else(get_mixed);

        expander::expand_union(context.codebase, &mut constant_type, &TypeExpansionOptions::default());

        artifacts.set_expression_type(self, constant_type);

        Ok(())
    }
}
