use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::psl::type_component::TypeComponentFunctionsHandler;
use crate::invocation::special_function_like_handler::standard::error::ErrorFunctionsHandler;
use crate::invocation::special_function_like_handler::standard::string::StringFunctionsHandler;

mod psl;
mod standard;
mod utils;

trait SpecialFunctionLikeHandlerTrait {
    fn get_return_type<'a>(
        &self,
        context: &mut Context<'a>,
        artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        invocation: &Invocation,
    ) -> Option<TUnion>;
}

pub fn handle_special_functions(
    context: &mut Context<'_>,
    artifacts: &AnalysisArtifacts,
    invocation: &Invocation,
) -> Option<TUnion> {
    const HANDLERS: &[&dyn SpecialFunctionLikeHandlerTrait] = &[
        // Standard PHP function handlers
        &StringFunctionsHandler,
        &ErrorFunctionsHandler,
        // PSL specific function handlers
        &TypeComponentFunctionsHandler,
    ];

    let function_like_identifier = invocation.target.get_function_like_identifier()?;
    let function_like_name = function_like_identifier.as_string(context.interner).to_lowercase();

    for handler in HANDLERS {
        if let Some(return_type) = handler.get_return_type(context, artifacts, &function_like_name, invocation) {
            return Some(return_type);
        }
    }

    None
}
