use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::r#loop;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for For<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let infinite_loop = self.initializations.is_empty() && self.conditions.is_empty() && self.increments.is_empty();

        r#loop::analyze_for_or_while_loop(
            context,
            block_context,
            artifacts,
            self.initializations.as_slice(),
            self.conditions.as_slice(),
            self.increments.as_slice(),
            self.body.statements(),
            self.span(),
            infinite_loop,
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_analysis;

    test_analysis! {
        name = for_loop_is_entered_al_least_once,
        code = indoc! {r#"
            <?php

            /**
             * @template T
             *
             * @param int<1, max> $size
             * @param (Closure(int): T) $factory
             *
             * @return non-empty-list<T>
             */
            function reproduce(int $size, Closure $factory): array {
                $result = [];
                for ($i = 1; $i <= $size; $i++) {
                    $result[] = $factory($i);
                }

                return $result;
            }
        "#},
    }
}
