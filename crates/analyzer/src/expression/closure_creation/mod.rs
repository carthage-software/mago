use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

pub mod function_closure_creation;
pub mod method_closure_creation;
pub mod static_method_closure_creation;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ClosureCreation<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match self {
            ClosureCreation::Function(function_closure_creation) => {
                function_closure_creation.analyze(context, block_context, artifacts)
            }
            ClosureCreation::Method(method_closure_creation) => {
                method_closure_creation.analyze(context, block_context, artifacts)
            }
            ClosureCreation::StaticMethod(static_method_closure_creation) => {
                static_method_closure_creation.analyze(context, block_context, artifacts)
            }
        }
    }
}
