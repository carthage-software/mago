//! `pow()` return type provider: the same result as the `**` operator for int and float operands.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::union::TUnion;

use crate::expression::binary::arithmetic::integer_power_result;
use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::math::pow", "pow", "Returns int or float like `**` when both operands are numbers");

#[derive(Default)]
pub struct PowProvider;

impl Provider for PowProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for PowProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact(b"pow")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let base = context.get_expression_type(invocation.get_argument(0, &[b"num"])?)?;
        let exponent = context.get_expression_type(invocation.get_argument(1, &[b"exponent"])?)?;

        let mut result = vec![];
        for base_atomic in base.types.iter() {
            for exponent_atomic in exponent.types.iter() {
                match (base_atomic, exponent_atomic) {
                    (TAtomic::Scalar(TScalar::Integer(base)), TAtomic::Scalar(TScalar::Integer(exponent))) => {
                        let literal =
                            base.get_literal_value().zip(exponent.get_literal_value()).and_then(|(base, exponent)| {
                                u32::try_from(exponent).ok().and_then(|exponent| base.checked_pow(exponent))
                            });

                        match literal {
                            Some(value) => result.push(TAtomic::Scalar(TScalar::literal_int(value))),
                            None => result.extend(integer_power_result(*base, *exponent)),
                        }
                    }
                    (
                        TAtomic::Scalar(TScalar::Integer(_) | TScalar::Float(_)),
                        TAtomic::Scalar(TScalar::Integer(_) | TScalar::Float(_)),
                    ) => result.push(TAtomic::Scalar(TScalar::float())),
                    _ => return None,
                }
            }
        }

        (!result.is_empty()).then(|| TUnion::from_vec(result))
    }
}
