use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::get_array_value_parameter;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
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
        FunctionTarget::Exact("min")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        if invocation.argument_count() > 1 {
            return None;
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

        if let Some(known_resulting_type) = &resulting_type {
            let mut possibilities = vec![];
            for atomic in known_resulting_type.types.iter() {
                let TAtomic::Scalar(TScalar::Integer(integer)) = atomic else {
                    return resulting_type;
                };

                possibilities.push(*integer);
            }

            let mut min_lb = possibilities[0].get_minimum_value();
            let mut min_ub = possibilities[0].get_maximum_value();
            for t in possibilities.iter().skip(1) {
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
