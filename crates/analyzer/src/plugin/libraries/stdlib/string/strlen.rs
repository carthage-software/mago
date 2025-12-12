//! strlen() return type provider.

use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::string::strlen", "strlen", "Returns literal int for literal string");

#[derive(Default)]
pub struct StrlenProvider;

impl Provider for StrlenProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for StrlenProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("strlen")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let string_argument = invocation.get_argument(0, &["string"])?;
        let string_argument_type = context.get_expression_type(string_argument)?;
        let string_literal = string_argument_type.get_single_literal_string_value()?;

        Some(get_literal_int(string_literal.len() as i64))
    }
}
