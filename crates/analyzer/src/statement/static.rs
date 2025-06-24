use std::rc::Rc;

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
use crate::expression::assignment::check_docblock_type_incompatibility;
use crate::expression::assignment::get_type_from_var_docblock;
use crate::issue::TypingIssueKind;

impl Analyzable for Static {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if block_context.is_mutation_free() && context.settings.analyze_effects {
            context.buffer.report(
                TypingIssueKind::StaticInMutationFreeContext,
                Issue::error(
                    "Cannot declare `static` variables inside a mutation-free function or method."
                )
                .with_annotation(
                    Annotation::primary(self.span()).with_message("`static` variable declared here.")
                )
                .with_note(
                    "Static variables maintain state across function calls, which violates the mutation-free guarantee."
                )
                .with_help(
                    "Remove the `static` declaration or remove the mutation-free annotation from the enclosing function/method."
                ),
            );
        }

        for item in self.items.iter() {
            let variable = item.variable();
            let initial_value = item.value();

            let mut inferred_type = None;
            if let Some(initial_value) = initial_value {
                let was_inside_general_use = block_context.inside_general_use;
                block_context.inside_general_use = true;
                initial_value.analyze(context, block_context, artifacts)?;
                block_context.inside_general_use = was_inside_general_use;

                inferred_type = artifacts.get_rc_expression_type(initial_value).cloned();
            }

            let variable_id = context.interner.lookup(&variable.name);
            let variable_span = variable.span();

            let variable_type = match (
                inferred_type,
                get_type_from_var_docblock(
                    context,
                    block_context,
                    artifacts,
                    Some(variable_id),
                    variable_span,
                    self.items.len() == 1,
                ),
            ) {
                (Some(inferred_type), Some((docblock_type, docblock_type_span))) => {
                    check_docblock_type_incompatibility(
                        context,
                        Some(variable_id),
                        variable_span,
                        &inferred_type,
                        &docblock_type,
                        docblock_type_span,
                        initial_value,
                    );

                    Rc::new(docblock_type)
                }
                (None, Some((docblock_type, _))) => Rc::new(docblock_type),
                (Some(inferred_type), None) => inferred_type,
                (None, None) => Rc::new(get_mixed()),
            };

            block_context.locals.insert(variable_id.to_owned(), variable_type);
            block_context.assigned_variable_ids.insert(variable_id.to_owned(), item.span().start.offset);
            block_context.static_locals.insert(variable_id.to_owned());
        }

        Ok(())
    }
}
