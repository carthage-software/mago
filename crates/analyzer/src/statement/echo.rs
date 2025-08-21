use mago_codex::ttype::get_nullable_scalar;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::common::construct::ConstructInput;
use crate::common::construct::analyze_construct_inputs;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Echo<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_construct_inputs(
            context,
            block_context,
            artifacts,
            "echo",
            self.echo.span,
            ConstructInput::ExpressionList(self.values.as_slice()),
            get_nullable_scalar(),
            true,
            false,
            true,
        )?;

        Ok(())
    }
}
