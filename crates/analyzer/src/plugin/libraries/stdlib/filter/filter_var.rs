//! `filter_var()` return type provider.

use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

use super::common;

static META: ProviderMeta = ProviderMeta::new(
    "php::filter::filter_var",
    "filter_var",
    "Returns narrowed type based on the filter validation constant",
);

/// Provider for the `filter_var()` function.
///
/// `filter_var(mixed $value, int $filter = FILTER_DEFAULT, array|int $options = 0): mixed`
#[derive(Default)]
pub struct FilterVarProvider;

impl Provider for FilterVarProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for FilterVarProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("filter_var")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        // filter_var($value, $filter, $options)
        //            0       1        2
        common::resolve_filter_return_type(context, invocation, 1, 2)
    }
}
