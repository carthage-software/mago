use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use mago_allocator::Arena;

pub(crate) trait Analyzable<'ast, 'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena;
}

impl<'ast, 'arena, T> Analyzable<'ast, 'arena> for Box<T>
where
    T: Analyzable<'ast, 'arena>,
{
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        (**self).analyze(context, block_context, artifacts)
    }
}

impl<'ast, 'arena, T> Analyzable<'ast, 'arena> for &T
where
    T: Analyzable<'ast, 'arena>,
{
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        (*self).analyze(context, block_context, artifacts)
    }
}

impl<'ast, 'arena, T> Analyzable<'ast, 'arena> for Option<T>
where
    T: Analyzable<'ast, 'arena>,
{
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        if let Some(inner) = self { inner.analyze(context, block_context, artifacts) } else { Ok(()) }
    }
}
