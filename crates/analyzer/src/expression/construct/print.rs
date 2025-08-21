use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::get_string;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::common::construct::ConstructInput;
use crate::common::construct::analyze_construct_inputs;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for PrintConstruct<'arena> {
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
            "print",
            self.print.span,
            ConstructInput::Expression(self.value),
            get_string(),
            false,
            false,
            true,
        )?;

        artifacts.set_expression_type(self, get_literal_int(1));

        Ok(())
    }
}
