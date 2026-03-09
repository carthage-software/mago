//! `Psl\Type\int_range()` return type provider.

use mago_atom::atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "psl::type::int_range",
    "Psl\\Type\\int_range",
    "Returns TypeInterface with int range inner type",
);

/// Provider for the `Psl\Type\int_range()` function.
///
/// Narrows the return type from `TypeInterface<int>` to `TypeInterface<int<min, max>>`
/// when the min and/or max arguments are known literal integers or integer ranges.
#[derive(Default)]
pub struct IntRangeProvider;

impl Provider for IntRangeProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for IntRangeProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("psl\\type\\int_range")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let min_arg = invocation.get_argument(0, &["min"])?;
        let max_arg = invocation.get_argument(1, &["max"])?;

        let min_type = context.get_expression_type(min_arg)?;
        let max_type = context.get_expression_type(max_arg)?;

        let min_bound = extract_minimum_bound(min_type);
        let max_bound = extract_maximum_bound(max_type);

        // If we can't determine either bound, fall back to the default signature.
        if min_bound.is_none() && max_bound.is_none() {
            return None;
        }

        let inner = TInteger::from_bounds(min_bound, max_bound);
        let inner_type = TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(inner)));

        Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            atom("Psl\\Type\\TypeInterface"),
            Some(vec![inner_type]),
        )))))
    }
}

/// Extract the overall minimum bound from a union of integer types.
///
/// For example, `1|5|int<3, 10>` has minimum bound `1` (the smallest lower bound).
fn extract_minimum_bound(union: &TUnion) -> Option<i64> {
    let mut result: Option<i64> = None;

    for atomic in union.types.iter() {
        let TAtomic::Scalar(TScalar::Integer(integer)) = atomic else {
            return None;
        };

        match integer.get_minimum_value() {
            Some(val) => {
                result = Some(result.map_or(val, |current: i64| current.min(val)));
            }
            None => return None,
        }
    }

    result
}

/// Extract the overall maximum bound from a union of integer types.
///
/// For example, `1|5|int<3, 10>` has maximum bound `10` (the largest upper bound).
fn extract_maximum_bound(union: &TUnion) -> Option<i64> {
    let mut result: Option<i64> = None;

    for atomic in union.types.iter() {
        let TAtomic::Scalar(TScalar::Integer(integer)) = atomic else {
            return None;
        };

        match integer.get_maximum_value() {
            Some(val) => {
                result = Some(result.map_or(val, |current: i64| current.max(val)));
            }
            None => return None,
        }
    }

    result
}
