//! rand() and mt_rand() return type provider.

use mago_codex::ttype::get_int_range;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::random::rand", "rand/mt_rand", "Returns int range based on min/max arguments");

static TARGETS: [&str; 2] = ["rand", "mt_rand"];

/// Provider for the `rand()` and `mt_rand()` functions.
///
/// Returns an int range based on the min/max arguments.
#[derive(Default)]
pub struct RandProvider;

impl Provider for RandProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for RandProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::ExactMultiple(&TARGETS)
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        if invocation.has_no_arguments() {
            return Some(get_int_range(Some(0), None));
        }

        let min_argument = invocation.get_argument(0, &["min"])?;
        let min_argument_type = context.get_expression_type(min_argument)?;
        let min_argument_integer = min_argument_type.get_single_int()?;

        let max_argument = invocation.get_argument(1, &["max"])?;
        let max_argument_type = context.get_expression_type(max_argument)?;
        let max_argument_integer = max_argument_type.get_single_int()?;

        let minimum_value = min_argument_integer.get_minimum_value()?;
        let maximum_value = max_argument_integer.get_maximum_value();

        Some(get_int_range(Some(minimum_value), maximum_value))
    }
}
