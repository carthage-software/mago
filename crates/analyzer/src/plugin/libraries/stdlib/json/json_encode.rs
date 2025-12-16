//! `json_encode()` return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::bool::TBool;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

const JSON_THROW_ON_ERROR: i64 = 4194304;

static META: ProviderMeta = ProviderMeta::new(
    "php::json::json_encode",
    "json_encode",
    "Returns non-empty-string when JSON_THROW_ON_ERROR is set, otherwise string|false",
);

/// Provider for the `json_encode()` function.
///
/// When `JSON_THROW_ON_ERROR` flag is set, returns `non-empty-string`.
/// Otherwise returns `non-empty-string|false`.
#[derive(Default)]
pub struct JsonEncodeProvider;

impl Provider for JsonEncodeProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for JsonEncodeProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("json_encode")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let flags_argument = invocation.get_argument(1, &["flags"])?;
        let flags_type = context.get_expression_type(flags_argument)?;
        let flags_value = flags_type.get_single_literal_int_value()?;

        Some(if flags_value & JSON_THROW_ON_ERROR > 0 {
            get_non_empty_string()
        } else {
            TUnion::from_vec(vec![
                TAtomic::Scalar(TScalar::String(TString::non_empty())),
                TAtomic::Scalar(TScalar::Bool(TBool::r#false())),
            ])
        })
    }
}
