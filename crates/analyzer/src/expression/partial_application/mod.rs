use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

pub mod function_partial_application;
pub mod method_partial_application;
pub mod static_method_partial_application;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for PartialApplication<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match self {
            PartialApplication::Function(function_partial_application) => {
                function_partial_application.analyze(context, block_context, artifacts)
            }
            PartialApplication::Method(method_partial_application) => {
                method_partial_application.analyze(context, block_context, artifacts)
            }
            PartialApplication::StaticMethod(static_method_partial_application) => {
                static_method_partial_application.analyze(context, block_context, artifacts)
            }
        }
    }
}
