use mago_codex::ttype::get_int_or_string;
use mago_codex::ttype::get_never;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::common::construct::ConstructInput;
use crate::common::construct::analyze_construct_inputs;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::control_action::ControlAction;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ExitConstruct<'arena> {
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
            "exit",
            self.exit.span,
            ConstructInput::ArgumentList(self.arguments.as_ref()),
            get_int_or_string(),
            true,
            true,
            true,
        )?;

        block_context.has_returned = true;
        block_context.control_actions.insert(ControlAction::End);

        artifacts.set_expression_type(self, get_never());

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for DieConstruct<'arena> {
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
            "die",
            self.die.span,
            ConstructInput::ArgumentList(self.arguments.as_ref()),
            get_int_or_string(),
            true,
            true,
            true,
        )?;

        block_context.has_returned = true;
        block_context.control_actions.insert(ControlAction::End);

        artifacts.set_expression_type(self, get_never());

        Ok(())
    }
}
