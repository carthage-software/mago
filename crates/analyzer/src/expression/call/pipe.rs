use foldhash::HashMap;

use mago_allocator::Arena;
use mago_codex::ttype::template::TemplateResult;
use mago_span::HasSpan;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Pipe;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::call::analyze_invocation_targets;
use crate::expression::call::function_call::resolve_targets;
use crate::invocation::InvocationArgumentsSource;
use crate::utils::misc::unwrap_expression;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Pipe<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        let mut template_result = TemplateResult::default();

        let callable = unwrap_expression(self.callable);
        let input_is_analyzed = matches!(callable, Expression::Closure(_) | Expression::ArrowFunction(_))
            && context
                .codebase
                .get_closure_at(context.source_file, callable.span())
                .and_then(|metadata| metadata.parameters.first())
                .is_some_and(|parameter| {
                    parameter.closure_this_type.is_none()
                        && parameter.get_type_metadata().is_none_or(|metadata| {
                            metadata.type_union.is_vanilla_mixed() || metadata.type_union.is_vanilla_array()
                        })
                });
        let inferred_parameter_types = artifacts.inferred_parameter_types.take();

        if input_is_analyzed {
            // Untyped or broad inline parameters need the input type before body analysis.
            // Keep declared callback signatures and bindings available to infer closure-valued inputs instead.
            let was_inside_call = block_context.flags.inside_call();
            let was_inside_general_use = block_context.flags.inside_general_use();
            block_context.flags.set_inside_call(true);
            block_context.flags.set_inside_general_use(true);
            let result = self.input.analyze(context, block_context, artifacts);
            block_context.flags.set_inside_call(was_inside_call);
            block_context.flags.set_inside_general_use(was_inside_general_use);

            if let Err(error) = result {
                artifacts.inferred_parameter_types = inferred_parameter_types;
                return Err(error);
            }

            artifacts.inferred_parameter_types = artifacts
                .get_expression_type(self.input)
                .map(|input_type| HashMap::from_iter([(0, input_type.clone())]));
        }

        let was_inside_pipe_callable = block_context.flags.inside_pipe_callable();
        block_context.flags.set_inside_pipe_callable(true);
        let result = resolve_targets(context, block_context, artifacts, self.callable, &mut template_result);
        block_context.flags.set_inside_pipe_callable(was_inside_pipe_callable);
        artifacts.inferred_parameter_types = inferred_parameter_types;
        let (invocation_targets, _) = result?;

        analyze_invocation_targets(
            context,
            block_context,
            artifacts,
            template_result,
            invocation_targets,
            InvocationArgumentsSource::PipeInput { pipe: self, input_is_analyzed },
            self.span(),
            None,
            false,
            false,
            false, // object_has_nullsafe_null - not applicable for pipe expressions
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = pipe_expression_too_many_args,
        code = indoc! {r#"
            <?php

            function do_nothing(): void { return; }

            "foo" |> do_nothing(...);
        "#},
        issues = [
            IssueCode::TooManyArguments,
        ],
    }

    test_analysis! {
        name = pipe_expression_too_few_args,
        code = indoc! {r#"
            <?php

            function do_nothing(int $_a, int $_b): void { return; }

            "foo" |> do_nothing(...);
        "#},
        issues = [
            IssueCode::InvalidArgument, // "foo" is not an int
            IssueCode::TooFewArguments,
        ],
    }

    test_analysis! {
        name = pipe_expression_exact_args,
        code = indoc! {"
            <?php

            function do_nothing(int $_a): void { return; }

            123 |> do_nothing(...);
        "},
    }
}
