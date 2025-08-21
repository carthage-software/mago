use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ClassLikeConstant<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::ClassLikeConstant,
        )?;

        for item in self.items.iter() {
            item.analyze(context, block_context, artifacts)?;
        }

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ClassLikeConstantItem<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        self.value.analyze(context, block_context, artifacts)?;

        Ok(())
    }
}
