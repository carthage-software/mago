use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::SpecialFunctionLikeHandlerTrait;
use crate::invocation::special_function_like_handler::utils::get_argument;

#[derive(Debug)]
pub struct StrComponentFunctionsHandler;

impl SpecialFunctionLikeHandlerTrait for StrComponentFunctionsHandler {
    fn get_return_type<'ctx, 'ast, 'arena>(
        &self,
        _context: &mut Context<'ctx, 'arena>,
        _block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<TUnion> {
        match function_like_name {
            "psl\\str\\after"
            | "psl\\str\\after_ci"
            | "psl\\str\\after_last"
            | "psl\\str\\after_last_ci"
            | "psl\\str\\before"
            | "psl\\str\\before_ci"
            | "psl\\str\\before_last"
            | "psl\\str\\before_last_ci"
            | "psl\\str\\byte\\after"
            | "psl\\str\\byte\\after_ci"
            | "psl\\str\\byte\\after_last"
            | "psl\\str\\byte\\after_last_ci"
            | "psl\\str\\byte\\before"
            | "psl\\str\\byte\\before_ci"
            | "psl\\str\\byte\\before_last"
            | "psl\\str\\byte\\before_last_ci"
            | "psl\\str\\grapheme\\after"
            | "psl\\str\\grapheme\\after_ci"
            | "psl\\str\\grapheme\\after_last"
            | "psl\\str\\grapheme\\after_last_ci"
            | "psl\\str\\grapheme\\before"
            | "psl\\str\\grapheme\\before_ci"
            | "psl\\str\\grapheme\\before_last"
            | "psl\\str\\grapheme\\before_last_ci" => {
                let haystack = get_argument(invocation.arguments_source, 0, vec!["haystack"])?;
                let haystack_type = artifacts.get_expression_type(haystack)?.get_single_string()?;

                Some(TUnion::from_vec(vec![
                    TAtomic::Null,
                    TAtomic::Scalar(TScalar::String(TString::general_with_props(
                        false,
                        false,
                        false,
                        haystack_type.is_lowercase,
                    ))),
                ]))
            }
            "psl\\str\\slice"
            | "psl\\str\\strip_prefix"
            | "psl\\str\\strip_suffix"
            | "psl\\str\\reverse"
            | "psl\\str\\trim"
            | "psl\\str\\trim_left"
            | "psl\\str\\trim_right"
            | "psl\\str\\truncate"
            | "psl\\str\\byte\\slice"
            | "psl\\str\\byte\\strip_prefix"
            | "psl\\str\\byte\\strip_suffix"
            | "psl\\str\\byte\\reverse"
            | "psl\\str\\byte\\trim"
            | "psl\\str\\byte\\trim_left"
            | "psl\\str\\byte\\trim_right"
            | "psl\\str\\grapheme\\slice"
            | "psl\\str\\grapheme\\strip_prefix"
            | "psl\\str\\grapheme\\strip_suffix"
            | "psl\\str\\grapheme\\reverse"
            | "psl\\str\\grapheme\\trim"
            | "psl\\str\\grapheme\\trim_left"
            | "psl\\str\\grapheme\\trim_right" => {
                let string = get_argument(invocation.arguments_source, 0, vec!["string"])?;
                let string_type = artifacts.get_expression_type(string)?.get_single_string()?;

                Some(if string_type.is_literal_origin() {
                    TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
                        false,
                        false,
                        false,
                        string_type.is_lowercase,
                    ))))
                } else {
                    TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                        false,
                        false,
                        false,
                        string_type.is_lowercase,
                    ))))
                })
            }
            "psl\\str\\splice" | "psl\\str\\byte\\splice" | "psl\\str\\grapheme\\splice" => {
                let string = get_argument(invocation.arguments_source, 0, vec!["string"])?;
                let replacement = get_argument(invocation.arguments_source, 1, vec!["replacement"])?;

                let string_type = artifacts.get_expression_type(string)?.get_single_string()?;
                let replacement_type = artifacts.get_expression_type(replacement)?.get_single_string()?;

                Some(if string_type.is_literal_origin() && replacement_type.is_literal_origin() {
                    TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
                        false,
                        string_type.is_truthy || replacement_type.is_truthy,
                        string_type.is_non_empty || replacement_type.is_non_empty,
                        string_type.is_lowercase && replacement_type.is_lowercase,
                    ))))
                } else {
                    TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                        false,
                        string_type.is_truthy || replacement_type.is_truthy,
                        string_type.is_non_empty || replacement_type.is_non_empty,
                        string_type.is_lowercase && replacement_type.is_lowercase,
                    ))))
                })
            }
            "psl\\str\\lowercase" | "psl\\str\\byte\\lowercase" | "psl\\str\\grapheme\\lowercase" => {
                let string = get_argument(invocation.arguments_source, 0, vec!["string"])?;
                let string_type = artifacts.get_expression_type(string)?.get_single_string()?;

                Some(match string_type.literal {
                    Some(_) => {
                        TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
                            string_type.is_numeric,
                            string_type.is_truthy,
                            string_type.is_non_empty,
                            true,
                        ))))
                    }
                    None => TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                        string_type.is_numeric,
                        string_type.is_truthy,
                        string_type.is_non_empty,
                        true,
                    )))),
                })
            }
            "psl\\str\\uppercase" | "psl\\str\\byte\\uppercase" | "psl\\str\\grapheme\\uppercase" => {
                let string = get_argument(invocation.arguments_source, 0, vec!["string"])?;
                let string_type = artifacts.get_expression_type(string)?.get_single_string()?;

                Some(match string_type.literal {
                    Some(_) => {
                        TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
                            string_type.is_numeric,
                            string_type.is_truthy,
                            string_type.is_non_empty,
                            false,
                        ))))
                    }
                    None => TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                        string_type.is_numeric,
                        string_type.is_truthy,
                        string_type.is_non_empty,
                        false,
                    )))),
                })
            }
            _ => None,
        }
    }
}
