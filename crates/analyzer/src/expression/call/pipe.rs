use mago_codex::ttype::template::TemplateResult;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::call::analyze_invocation_targets;
use crate::expression::call::function_call::resolve_targets;
use crate::invocation::InvocationArgumentsSource;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Pipe<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let mut template_result = TemplateResult::default();

        let (invocation_targets, encountered_invalid_targets) =
            resolve_targets(context, block_context, artifacts, self.callable, &mut template_result)?;

        analyze_invocation_targets(
            context,
            block_context,
            artifacts,
            template_result,
            invocation_targets,
            InvocationArgumentsSource::PipeInput(self),
            self.span(),
            encountered_invalid_targets,
            false,
            false,
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
        code = indoc! {r#"
            <?php

            function do_nothing(int $_a): void { return; }

            123 |> do_nothing(...);
        "#},
    }
}
