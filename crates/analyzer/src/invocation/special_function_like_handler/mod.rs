use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::core::closure::GetCurrentClosureMethodHandler;
use crate::invocation::special_function_like_handler::psl::type_component::TypeComponentFunctionsHandler;
use crate::invocation::special_function_like_handler::random::RandomFunctionsHandler;
use crate::invocation::special_function_like_handler::standard::error::ErrorFunctionsHandler;
use crate::invocation::special_function_like_handler::standard::string::StringFunctionsHandler;

mod core;
mod psl;
mod random;
mod standard;
mod utils;

trait SpecialFunctionLikeHandlerTrait {
    fn get_return_type<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &BlockContext<'a>,
        artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        invocation: &Invocation,
    ) -> Option<TUnion>;
}

pub fn handle_special_functions<'a>(
    context: &mut Context<'a>,
    block_context: &BlockContext<'a>,
    artifacts: &AnalysisArtifacts,
    invocation: &Invocation,
) -> Option<TUnion> {
    const HANDLERS: &[&dyn SpecialFunctionLikeHandlerTrait] = &[
        // Core function handlers
        &GetCurrentClosureMethodHandler,
        // Standard PHP function handlers
        &StringFunctionsHandler,
        &ErrorFunctionsHandler,
        // Random extension function handlers
        &RandomFunctionsHandler,
        // PSL specific function handlers
        &TypeComponentFunctionsHandler,
    ];

    let function_like_identifier = invocation.target.get_function_like_identifier()?;
    let name = function_like_identifier.as_string(context.interner).to_lowercase();

    for handler in HANDLERS {
        if let Some(return_type) = handler.get_return_type(context, block_context, artifacts, &name, invocation) {
            return Some(return_type);
        }
    }

    None
}
