use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_false;
use mago_codex::ttype::get_true;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for EmptyConstruct<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let was_inside_isset = block_context.inside_isset;
        block_context.inside_isset = true;
        self.value.analyze(context, block_context, artifacts)?;
        block_context.inside_isset = was_inside_isset;

        artifacts.set_expression_type(
            self,
            match artifacts.get_expression_type(&self.value) {
                Some(value_type) if value_type.is_always_truthy() => get_true(),
                Some(value_type) if value_type.is_always_falsy() => get_false(),
                _ => get_bool(),
            },
        );

        Ok(())
    }
}
