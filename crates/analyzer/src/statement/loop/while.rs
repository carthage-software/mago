use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::r#loop;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for While<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let is_while_true = match self.condition {
            Expression::Literal(literal) => match literal {
                Literal::True(_) => true,
                Literal::Integer(integer) => integer.value.is_none_or(|v| v > 0),
                _ => false,
            },
            _ => false,
        };

        r#loop::analyze_for_or_while_loop(
            context,
            block_context,
            artifacts,
            &[],
            std::slice::from_ref(self.condition),
            &[],
            self.body.statements(),
            self.span(),
            is_while_true,
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = condition_is_too_complex,
        code = indoc! {r#"
            <?php

            function is_special_case(int $id, int $count, float $score, float $threshold, bool $is_active, bool $is_admin, string $name, string $role, string $permission, string $category): bool {
                while(
                    ($id > 1000 && $count < 5 || $score >= 99.5 && $threshold < $score || $name === 'azjezz' && $role !== 'guest') &&
                    ($is_active && !$is_admin || $permission === 'write' && ($category === 'critical' || $category === 'urgent')) ||
                    !($count === 0 || $id < 0) && (
                        $role === 'admin' && $is_admin ||
                        $name !== 'guest' && $permission !== 'none' ||
                        ($score - $threshold) > 5.0 && $count > 1
                    ) && (
                        $category === 'general' || $category === 'special' ||
                        ($is_active && $is_admin && $id % 2 === 0) ||
                        ($name !== 'system' && $role !== 'user' && $score < 50.0)
                    ) || (
                        $id < 0 && $count > 100 ||
                        ($score < 10.0 && $threshold > 20.0) ||
                        ($is_active && $is_admin && $name === 'root') ||
                        ($role === 'guest' && $permission === 'read' && $category === 'public')
                    )
                ) {
                    // Do something
                }

                return true;
            }
        "#},
        issues = [
            IssueCode::ConditionIsTooComplex,
        ],
    }
}
