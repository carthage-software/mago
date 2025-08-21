use std::rc::Rc;

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
use crate::context::block::ReferenceConstraint;
use crate::context::block::ReferenceConstraintSource;
use crate::error::AnalysisError;
use crate::utils::docblock::check_docblock_type_incompatibility;
use crate::utils::docblock::get_type_from_var_docblock;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Static<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if block_context.scope.is_pure() {
            context.collector.report_with_code(
                IssueCode::ImpureStaticVariable,
                Issue::error(
                    "Cannot declare `static` variables inside a pure function or method."
                )
                .with_annotation(
                    Annotation::primary(self.span()).with_message("`static` variable declared here.")
                )
                .with_note(
                    "Static variables maintain state across function calls, which violates the pure guarantee."
                )
                .with_help(
                    "Remove the `static` declaration or remove the `@pure` annotation from the enclosing function/method."
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

            let variable_span = variable.span();

            let docblock_type = get_type_from_var_docblock(
                context,
                block_context,
                artifacts,
                Some(variable.name),
                self.items.len() == 1,
            );

            let variable_type = match (inferred_type, docblock_type) {
                (Some(inferred_type), Some((docblock_type, docblock_type_span))) => {
                    block_context.by_reference_constraints.insert(
                        variable.name.to_owned(),
                        ReferenceConstraint::new(
                            docblock_type_span,
                            ReferenceConstraintSource::Static,
                            Some(docblock_type.clone()),
                        ),
                    );

                    check_docblock_type_incompatibility(
                        context,
                        Some(variable.name),
                        variable_span,
                        &inferred_type,
                        &docblock_type,
                        docblock_type_span,
                        initial_value,
                    );

                    Rc::new(docblock_type)
                }
                (None, Some((docblock_type, docblock_type_span))) => {
                    block_context.by_reference_constraints.insert(
                        variable.name.to_owned(),
                        ReferenceConstraint::new(
                            docblock_type_span,
                            ReferenceConstraintSource::Static,
                            Some(docblock_type.clone()),
                        ),
                    );

                    Rc::new(docblock_type)
                }
                (Some(inferred_type), None) => inferred_type,
                (None, None) => Rc::new(get_mixed()),
            };

            block_context.locals.insert(variable.name.to_owned(), variable_type);
            block_context.assigned_variable_ids.insert(variable.name.to_owned(), item.span().start.offset);
            block_context.static_locals.insert(variable.name.to_owned());
        }

        Ok(())
    }
}
