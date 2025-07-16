use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::SpecialFunctionLikeHandlerTrait;

#[derive(Debug)]
pub struct ErrorFunctionsHandler;

impl SpecialFunctionLikeHandlerTrait for ErrorFunctionsHandler {
    fn get_return_type<'a>(
        &self,
        context: &mut Context<'a>,
        _artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        _invocation: &Invocation,
    ) -> Option<TUnion> {
        match function_like_name {
            "trigger_error" => {
                if context.settings.trigger_error_exists {
                    return Some(get_never());
                }

                Some(get_bool())
            }
            _ => None,
        }
    }
}
