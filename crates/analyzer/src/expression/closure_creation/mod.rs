use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

pub mod function_closure_creation;
pub mod method_closure_creation;
pub mod static_method_closure_creation;

impl Analyzable for ClosureCreation {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
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
