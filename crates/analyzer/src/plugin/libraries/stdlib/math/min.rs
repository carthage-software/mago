use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::get_array_value_parameter;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::libraries::stdlib::math::collect_integers;
use crate::plugin::libraries::stdlib::math::get_integer_from_type;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

/// Provider for the `min()` function.
#[derive(Default)]
pub struct MinProvider;

impl Provider for MinProvider {
    fn meta() -> &'static ProviderMeta {
        static META: ProviderMeta = ProviderMeta::new(
            "php::math::min",
            "min",
            "Return the minimum value of the values in the array passed as an argument.",
        );

        &META
    }
}

impl FunctionReturnTypeProvider for MinProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::ExactMultiple(&["min", "psl\\math\\min", "psl\\math\\minva"])
    }

    #[allow(clippy::similar_names)]
    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let arg_count = invocation.argument_count();
        if arg_count >= 2 {
            return get_min_of_args(context, invocation);
        }

        let value = invocation.get_argument(0, &["value"])?;
        let value_type = context.get_expression_type(value)?;

        let mut resulting_type = None;
        for atomic in value_type.types.iter() {
            let TAtomic::Array(array_type) = atomic else {
                return None;
            };

            resulting_type = Some(add_optional_union_type(
                get_array_value_parameter(array_type, context.codebase),
                resulting_type.as_ref(),
                context.codebase,
            ));
        }

        if let Some(known_resulting_type) = &resulting_type
            && let Some(integers) = collect_integers(known_resulting_type)
        {
            let mut min_lb = integers[0].get_minimum_value();
            let mut min_ub = integers[0].get_maximum_value();
            for t in integers.iter().skip(1) {
                let (lb, ub) = t.get_bounds();

                match (min_lb, lb) {
                    (Some(cur), Some(new)) => min_lb = Some(cur.min(new)),
                    (_, None) => min_lb = None,
                    _ => {}
                }

                match (min_ub, ub) {
                    (Some(cur), Some(new)) => min_ub = Some(cur.min(new)),
                    (None, Some(new)) => min_ub = Some(new),
                    _ => {}
                }
            }

            return Some(TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::from_bounds(min_lb, min_ub)))));
        }

        resulting_type
    }
}

#[allow(clippy::similar_names)]
fn get_min_of_args(context: &ProviderContext<'_, '_, '_>, invocation: &InvocationInfo<'_, '_, '_>) -> Option<TUnion> {
    let first = invocation.get_argument(0, &["value"])?;
    let first_type = context.get_expression_type(first)?;
    let mut result = get_integer_from_type(first_type)?;

    for i in 1..invocation.argument_count() {
        let arg = invocation.get_argument(i, &[])?;
        let arg_type = context.get_expression_type(arg)?;
        let integer = get_integer_from_type(arg_type)?;

        let (a_lb, a_ub) = result.get_bounds();
        let (b_lb, b_ub) = integer.get_bounds();

        let lb = match (a_lb, b_lb) {
            (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
            _ => None,
        };

        let ub = match (a_ub, b_ub) {
            (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        result = TInteger::from_bounds(lb, ub);
    }

    Some(TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(result))))
}
