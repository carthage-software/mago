use mago_codex::ttype::TType;
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
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Constant<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::Constant,
        )?;

        for item in self.items.iter() {
            let name = context.resolved_names.get(&item.name);

            let Some(constant_metadata) = context.codebase.get_constant(name) else {
                return Err(AnalysisError::InternalError(
                    format!("Constant metadata for `{name}` not found during analysis."),
                    item.name.span(),
                ));
            };

            item.value.analyze(context, block_context, artifacts)?;

            if constant_metadata.inferred_type.is_none() && constant_metadata.type_metadata.is_none() {
                if let Some(analysis_type) = artifacts.get_expression_type(&item.value) {
                    let analysis_type_str = analysis_type.get_id();

                    context.collector.report_with_code(
                        IssueCode::NonDocumentedConstant,
                        Issue::warning(format!(
                            "Type of constant `{name}` is not explicitly defined.",
                        ))
                        .with_annotation(
                            Annotation::primary(item.value.span())
                                .with_message(format!("The type was inferred as `{analysis_type_str}`, but an explicit `@var` tag is needed for reliable analysis.")),
                        )
                        .with_note(
                            "The type of a constant must be known before its usage is analyzed."
                        )
                        .with_note(
                            "Without a `@var` tag for this complex value, the constant is assumed to be `mixed`, which can hide potential type errors."
                        )
                        .with_help(format!(
                            "Add a docblock to make the type explicit: `/** @var {analysis_type_str} */`",
                        )),
                    );
                } else {
                    context.collector.report_with_code(
                        IssueCode::NonDocumentedConstant,
                        Issue::warning(format!("Could not determine the type of constant `{name}`."))
                            .with_annotation(
                                Annotation::primary(item.value.span())
                                    .with_message("The type of this expression could not be determined"),
                            )
                            .with_note("Without a `@var` tag, this constant will be treated as `mixed`.")
                            .with_help(
                                "Add a docblock to specify the constant's type, for example: `/** @var string */`",
                            ),
                    );
                }
            }
        }

        Ok(())
    }
}
