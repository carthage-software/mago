//! PSL string function return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::atomic::scalar::string::TStringCasing;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new("psl::str", "Psl\\Str\\*", "Returns refined string types based on input");

/// Provider for PSL string functions.
///
/// Provides refined string types based on the input string properties.
#[derive(Default)]
pub struct StrProvider;

impl Provider for StrProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for StrProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Namespace("psl\\str")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let function_name = invocation.function_name().to_lowercase();

        match function_name.as_str() {
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
                let haystack = invocation.get_argument(0, &["haystack"])?;
                let haystack_type = context.get_expression_type(haystack)?.get_single_string()?;

                Some(TUnion::from_vec(vec![
                    TAtomic::Null,
                    TAtomic::Scalar(TScalar::String(TString::general_with_props(
                        false,
                        false,
                        false,
                        haystack_type.casing,
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
                let string = invocation.get_argument(0, &["string"])?;
                let string_type = context.get_expression_type(string)?.get_single_string()?;

                Some(if string_type.is_literal_origin() {
                    TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
                        false,
                        false,
                        false,
                        string_type.casing,
                    ))))
                } else {
                    TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                        false,
                        false,
                        false,
                        string_type.casing,
                    ))))
                })
            }
            "psl\\str\\splice" | "psl\\str\\byte\\splice" | "psl\\str\\grapheme\\splice" => {
                let string = invocation.get_argument(0, &["string"])?;
                let replacement = invocation.get_argument(1, &["replacement"])?;

                let string_type = context.get_expression_type(string)?.get_single_string()?;
                let replacement_type = context.get_expression_type(replacement)?.get_single_string()?;

                Some(if string_type.is_literal_origin() && replacement_type.is_literal_origin() {
                    TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
                        false,
                        string_type.is_truthy || replacement_type.is_truthy,
                        string_type.is_non_empty || replacement_type.is_non_empty,
                        match (string_type.casing, replacement_type.casing) {
                            (TStringCasing::Lowercase, TStringCasing::Lowercase) => TStringCasing::Lowercase,
                            (TStringCasing::Uppercase, TStringCasing::Uppercase) => TStringCasing::Uppercase,
                            _ => TStringCasing::Unspecified,
                        },
                    ))))
                } else {
                    TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                        false,
                        string_type.is_truthy || replacement_type.is_truthy,
                        string_type.is_non_empty || replacement_type.is_non_empty,
                        match (string_type.casing, replacement_type.casing) {
                            (TStringCasing::Lowercase, TStringCasing::Lowercase) => TStringCasing::Lowercase,
                            (TStringCasing::Uppercase, TStringCasing::Uppercase) => TStringCasing::Uppercase,
                            _ => TStringCasing::Unspecified,
                        },
                    ))))
                })
            }
            "psl\\str\\lowercase" | "psl\\str\\byte\\lowercase" | "psl\\str\\grapheme\\lowercase" => {
                let string = invocation.get_argument(0, &["string"])?;
                let string_type = context.get_expression_type(string)?.get_single_string()?;

                Some(match string_type.literal {
                    Some(_) => {
                        TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
                            string_type.is_numeric,
                            string_type.is_truthy,
                            string_type.is_non_empty,
                            TStringCasing::Lowercase,
                        ))))
                    }
                    None => TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                        string_type.is_numeric,
                        string_type.is_truthy,
                        string_type.is_non_empty,
                        TStringCasing::Lowercase,
                    )))),
                })
            }
            "psl\\str\\uppercase" | "psl\\str\\byte\\uppercase" | "psl\\str\\grapheme\\uppercase" => {
                let string = invocation.get_argument(0, &["string"])?;
                let string_type = context.get_expression_type(string)?.get_single_string()?;

                Some(match string_type.literal {
                    Some(_) => {
                        TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
                            string_type.is_numeric,
                            string_type.is_truthy,
                            string_type.is_non_empty,
                            TStringCasing::Uppercase,
                        ))))
                    }
                    None => TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                        string_type.is_numeric,
                        string_type.is_truthy,
                        string_type.is_non_empty,
                        TStringCasing::Uppercase,
                    )))),
                })
            }
            _ => None,
        }
    }
}
