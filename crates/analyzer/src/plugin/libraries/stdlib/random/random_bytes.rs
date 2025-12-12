//! random_bytes() return type provider.

use mago_codex::ttype::get_empty_string;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::random::random_bytes", "random_bytes", "Returns non-empty-string when length > 0");

/// Provider for the `random_bytes()` function.
///
/// Returns `non-empty-string` when the length argument is > 0.
#[derive(Default)]
pub struct RandomBytesProvider;

impl Provider for RandomBytesProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for RandomBytesProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("random_bytes")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let length_argument = invocation.get_argument(0, &["length"])?;
        let length_argument_type = context.get_expression_type(length_argument)?;
        let length_argument_integer = length_argument_type.get_single_int()?;
        let minimum_value = length_argument_integer.get_minimum_value()?;

        Some(if minimum_value > 0 { get_non_empty_string() } else { get_empty_string() })
    }
}
