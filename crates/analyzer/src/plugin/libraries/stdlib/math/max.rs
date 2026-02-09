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

/// Provider for the `max()` function.
#[derive(Default)]
pub struct MaxProvider;

impl Provider for MaxProvider {
    fn meta() -> &'static ProviderMeta {
        static META: ProviderMeta = ProviderMeta::new(
            "php::math::max",
            "max",
            "Return the maximum value of the values in the array passed as an argument.",
        );

        &META
    }
}

impl FunctionReturnTypeProvider for MaxProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("max")
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

            let mut max_lb = possibilities[0].get_minimum_value();
            let mut max_ub = possibilities[0].get_maximum_value();
            for t in possibilities.iter().skip(1) {
                let (lb, ub) = t.get_bounds();

                max_lb = std::cmp::max(max_lb, lb);

                if max_ub.is_none() || ub.is_none() {
                    max_ub = None;
                } else {
                    max_ub = std::cmp::max(max_ub, ub);
                }
            }

            return Some(TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::from_bounds(max_lb, max_ub)))));
        }

        resulting_type
    }
}
