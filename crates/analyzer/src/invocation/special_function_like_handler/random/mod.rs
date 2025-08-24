use mago_codex::ttype::get_empty_string;
use mago_codex::ttype::get_int_range;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::SpecialFunctionLikeHandlerTrait;
use crate::invocation::special_function_like_handler::utils::get_argument;

#[derive(Debug)]
pub struct RandomFunctionsHandler;

impl SpecialFunctionLikeHandlerTrait for RandomFunctionsHandler {
    fn get_return_type<'ctx, 'ast, 'arena>(
        &self,
        _context: &mut Context<'ctx, 'arena>,
        _block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<TUnion> {
        match function_like_name {
            "rand" | "mt_rand" => {
                if invocation.arguments_source.get_arguments().is_empty() {
                    // `rand()` without arguments returns an int in the range [0, getrandmax()]
                    return Some(get_int_range(Some(0), None));
                }

                let min_argument = get_argument(invocation.arguments_source, 0, vec!["min"])?;
                let min_argument_type = artifacts.get_expression_type(min_argument)?;
                let min_argument_integer = min_argument_type.get_single_int()?;

                let max_argument = get_argument(invocation.arguments_source, 1, vec!["max"])?;
                let max_argument_type = artifacts.get_expression_type(max_argument)?;
                let max_argument_integer = max_argument_type.get_single_int()?;

                let minimum_value = min_argument_integer.get_minimum_value()?;
                let maximum_value = max_argument_integer.get_maximum_value();

                Some(get_int_range(Some(minimum_value), maximum_value))
            }
            "random_int" => {
                let min_argument = get_argument(invocation.arguments_source, 0, vec!["min"])?;
                let min_argument_type = artifacts.get_expression_type(min_argument)?;
                let min_argument_integer = min_argument_type.get_single_int()?;

                let max_argument = get_argument(invocation.arguments_source, 1, vec!["max"])?;
                let max_argument_type = artifacts.get_expression_type(max_argument)?;
                let max_argument_integer = max_argument_type.get_single_int()?;

                let minimum_value = min_argument_integer.get_minimum_value()?;
                let maximum_value = max_argument_integer.get_maximum_value();

                Some(get_int_range(Some(minimum_value), maximum_value))
            }
            "random_bytes" => {
                let length_argument = get_argument(invocation.arguments_source, 0, vec!["length"])?;
                let length_argument_type = artifacts.get_expression_type(length_argument)?;
                let length_argument_integer = length_argument_type.get_single_int()?;
                let minimum_value = length_argument_integer.get_minimum_value()?;

                Some(if minimum_value > 0 { get_non_empty_string() } else { get_empty_string() })
            }
            _ => None,
        }
    }
}
