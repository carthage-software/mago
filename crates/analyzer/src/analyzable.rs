use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

pub(crate) trait Analyzable {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>;
}

impl<T> Analyzable for Box<T>
where
    T: Analyzable,
{
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        (**self).analyze(context, block_context, artifacts)
    }
}

impl<T> Analyzable for &T
where
    T: Analyzable,
{
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        (**self).analyze(context, block_context, artifacts)
    }
}

impl<T> Analyzable for Option<T>
where
    T: Analyzable,
{
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if let Some(value) = self { value.analyze(context, block_context, artifacts) } else { Ok(()) }
    }
}
