use std::rc::Rc;

use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_mixed;
use mago_span::HasSpan;
use mago_syntax::ast::ArrayAccess;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::utils::expression::array::get_array_target_type_given_index;
use crate::utils::expression::get_array_access_id;
use crate::utils::expression::get_expression_id;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ArrayAccess<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let keyed_array_var_id = get_array_access_id(
            self,
            block_context.scope.get_class_like_name(),
            context.resolved_names,
            Some(context.codebase),
        );

        let extended_var_id = get_expression_id(
            self.array,
            block_context.scope.get_class_like_name(),
            context.resolved_names,
            Some(context.codebase),
        );

        let was_inside_use = block_context.inside_general_use;
        block_context.inside_general_use = true;
        block_context.inside_unset = false;
        self.index.analyze(context, block_context, artifacts)?;
        block_context.inside_general_use = was_inside_use;

        let index_type = artifacts.get_expression_type(&self.index).cloned().unwrap_or_else(get_arraykey);

        let was_inside_general_use = block_context.inside_general_use;
        block_context.inside_general_use = true;
        self.array.analyze(context, block_context, artifacts)?;
        block_context.inside_general_use = was_inside_general_use;

        if let Some(keyed_array_var_id) = &keyed_array_var_id
            && block_context.has_variable(keyed_array_var_id)
            && let Some(array_access_type) = block_context.locals.remove(keyed_array_var_id)
        {
            artifacts.set_rc_expression_type(self, array_access_type.clone());

            block_context.locals.insert(keyed_array_var_id.clone(), array_access_type.clone());

            return Ok(());
        }

        let container_type = artifacts.get_rc_expression_type(&self.array).cloned();

        if let Some(container_type) = container_type {
            let access_type = get_array_target_type_given_index(
                context,
                block_context,
                self.span(),
                self.array.span(),
                Some(self.index.span()),
                &container_type,
                &index_type,
                false,
                &extended_var_id,
            );

            if let Some(keyed_array_var_id) = &keyed_array_var_id {
                let can_store_result = block_context.inside_assignment || !container_type.is_mixed();

                if !block_context.inside_isset && can_store_result && keyed_array_var_id.contains("[$") {
                    block_context.locals.insert(keyed_array_var_id.clone(), Rc::new(access_type.clone()));
                }
            }

            artifacts.set_expression_type(self, access_type.clone());
        } else {
            artifacts.set_expression_type(self, get_mixed());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_analysis;

    test_analysis! {
        name = using_generic_parameter_as_index,
        code = indoc! {r#"
            <?php

            /**
             * @template T as string|lowercase-string
             *
             * @param array<T, bool> $old
             * @param array<T, bool> $new
             * @return array<T, bool>
             */
            function mergeThreadData(array $old, array $new): array {
                foreach ($new as $name => $value) {
                    if (!isset($old[$name]) || !$old[$name] && $value) {
                        $old[$name] = $value;
                    }
                }

                return $old;
            }
        "#},
    }
}
