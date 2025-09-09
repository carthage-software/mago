use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::core::closure::GetCurrentClosureMethodHandler;
use crate::invocation::special_function_like_handler::psl::regex_component::RegexComponentFunctionsHandler;
use crate::invocation::special_function_like_handler::psl::str_component::StrComponentFunctionsHandler;
use crate::invocation::special_function_like_handler::psl::type_component::TypeComponentFunctionsHandler;
use crate::invocation::special_function_like_handler::random::RandomFunctionsHandler;
use crate::invocation::special_function_like_handler::spl::iterator::IteratorFunctionsHandler;
use crate::invocation::special_function_like_handler::standard::array::ArrayFunctionsHandler;
use crate::invocation::special_function_like_handler::standard::string::StringFunctionsHandler;

mod core;
mod psl;
mod random;
mod spl;
mod standard;
mod utils;

trait SpecialFunctionLikeHandlerTrait {
    fn get_return_type<'ctx, 'ast, 'arena>(
        &self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<TUnion>;
}

pub fn handle_special_functions<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
) -> Option<TUnion> {
    const HANDLERS: &[&dyn SpecialFunctionLikeHandlerTrait] = &[
        // Core function handlers
        &GetCurrentClosureMethodHandler,
        // Standard PHP function handlers
        &StringFunctionsHandler,
        &ArrayFunctionsHandler,
        // SPL function handlers
        &IteratorFunctionsHandler,
        // Random extension function handlers
        &RandomFunctionsHandler,
        // PSL specific function handlers
        &StrComponentFunctionsHandler,
        &TypeComponentFunctionsHandler,
        &RegexComponentFunctionsHandler,
    ];

    let function_like_identifier = invocation.target.get_function_like_identifier()?;
    let name = function_like_identifier.as_string().to_lowercase();

    for handler in HANDLERS {
        if let Some(return_type) = handler.get_return_type(context, block_context, artifacts, &name, invocation) {
            return Some(return_type);
        }
    }

    None
}
