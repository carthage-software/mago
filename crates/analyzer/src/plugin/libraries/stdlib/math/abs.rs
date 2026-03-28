use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

use super::get_integer_from_type;

/// Provider for the `abs()` function.
#[derive(Default)]
pub struct AbsProvider;

impl Provider for AbsProvider {
    fn meta() -> &'static ProviderMeta {
        static META: ProviderMeta =
            ProviderMeta::new("php::math::abs", "abs", "Return the absolute value of a number.");

        &META
    }
}

impl FunctionReturnTypeProvider for AbsProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::ExactMultiple(&["abs", "psl\\math\\abs"])
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let arg = invocation.get_argument(0, &["num"])?;
        let arg_type = context.get_expression_type(arg)?;
        let integer = get_integer_from_type(arg_type)?;

        let (lb, ub) = integer.get_bounds();

        let result = match (lb, ub) {
            (Some(low), _) if low >= 0 => integer,
            (_, Some(high)) if high <= 0 => {
                let new_lb = high.checked_neg().map(Some).unwrap_or(None);
                let new_ub = lb.and_then(|v| v.checked_neg());

                TInteger::from_bounds(new_lb, new_ub)
            }
            (Some(low), Some(high)) => {
                let abs_low = low.checked_neg().unwrap_or(i64::MAX);
                let abs_high = high;

                TInteger::from_bounds(Some(0), Some(std::cmp::max(abs_low, abs_high)))
            }
            _ => TInteger::from_bounds(Some(0), None),
        };

        Some(TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(result))))
    }
}
