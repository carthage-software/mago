//! `filter_input()` return type provider.

use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

use super::common;

static META: ProviderMeta = ProviderMeta::new(
    "php::filter::filter_input",
    "filter_input",
    "Returns narrowed type based on the filter validation constant",
);

/// Provider for the `filter_input()` function.
///
/// `filter_input(int $type, string $var_name, int $filter = FILTER_DEFAULT, array|int $options = 0): mixed`
#[derive(Default)]
pub struct FilterInputProvider;

impl Provider for FilterInputProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for FilterInputProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("filter_input")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        // filter_input($type, $var_name, $filter, $options)
        //              0      1          2        3
        common::resolve_filter_return_type(context, invocation, 2, 3)
    }
}
