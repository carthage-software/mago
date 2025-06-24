use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::r#loop;

impl Analyzable for While {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let is_while_true = match self.condition.as_ref() {
            Expression::Literal(literal) => match literal {
                Literal::True(_) => true,
                Literal::Integer(integer) => integer.value > 0,
                _ => false,
            },
            _ => false,
        };

        r#loop::analyze_for_or_while_loop(
            context,
            block_context,
            artifacts,
            &[],
            std::slice::from_ref(&self.condition),
            &[],
            self.body.statements(),
            self.span(),
            is_while_true,
        )
    }
}
