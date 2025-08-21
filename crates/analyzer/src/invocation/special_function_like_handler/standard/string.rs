use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::SpecialFunctionLikeHandlerTrait;
use crate::invocation::special_function_like_handler::utils::get_argument;

#[derive(Debug)]
pub struct StringFunctionsHandler;

impl SpecialFunctionLikeHandlerTrait for StringFunctionsHandler {
    fn get_return_type<'ctx, 'ast, 'arena>(
        &self,
        _context: &mut Context<'ctx, 'arena>,
        _block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<TUnion> {
        match function_like_name {
            "strlen" => {
                let string_argument = get_argument(invocation.arguments_source, 0, vec!["string"])?;
                let string_argument_type = artifacts.get_expression_type(string_argument)?;
                let string_literal = string_argument_type.get_single_literal_string_value()?;

                Some(get_literal_int(string_literal.len() as i64))
            }
            _ => None,
        }
    }
}
