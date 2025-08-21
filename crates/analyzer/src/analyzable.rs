use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

pub(crate) trait Analyzable<'ast, 'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>;
}

impl<'ast, 'arena, T> Analyzable<'ast, 'arena> for Box<T>
where
    T: Analyzable<'ast, 'arena>,
{
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        (**self).analyze(context, block_context, artifacts)
    }
}

impl<'ast, 'arena, T> Analyzable<'ast, 'arena> for &T
where
    T: Analyzable<'ast, 'arena>,
{
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        (*self).analyze(context, block_context, artifacts)
    }
}

impl<'ast, 'arena, T> Analyzable<'ast, 'arena> for Option<T>
where
    T: Analyzable<'ast, 'arena>,
{
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if let Some(inner) = self { inner.analyze(context, block_context, artifacts) } else { Ok(()) }
    }
}
