use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::bool::TBool;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::SpecialFunctionLikeHandlerTrait;
use crate::invocation::special_function_like_handler::utils::get_argument;

const JSON_THROW_ON_ERROR: i64 = 4194304;

#[derive(Debug)]
pub struct JsonFunctionsHandler;

impl SpecialFunctionLikeHandlerTrait for JsonFunctionsHandler {
    fn get_return_type<'ctx, 'ast, 'arena>(
        &self,
        _context: &mut Context<'ctx, 'arena>,
        _block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<TUnion> {
        match function_like_name {
            "json_encode" => {
                let int_flags = get_argument(invocation.arguments_source, 1, vec!["flags"])?;
                let int_argument_type = artifacts.get_expression_type(int_flags)?;
                let int_literal = int_argument_type.get_single_literal_int_value()?;

                Some(if int_literal & JSON_THROW_ON_ERROR > 0 {
                    TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::new(
                        None,
                        false,
                        false,
                        true,
                        false,
                    ))))
                } else {
                    TUnion::from_vec(vec![
                        TAtomic::Scalar(TScalar::String(TString::new(
                            None,
                            false,
                            false,
                            true,
                            false,
                        ))),
                        TAtomic::Scalar(TScalar::Bool(TBool { value: Some(false) }))
                    ])
                })
            }
            _ => None,
        }
    }
}
