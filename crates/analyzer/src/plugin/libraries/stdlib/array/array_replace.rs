//! `array_replace()` return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::union::TUnion;

use crate::expression::binary::arithmetic::compose_keyed_plus;
use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::array::array_replace", "array_replace", "Returns the shape of the replaced array");

#[derive(Default)]
pub struct ArrayReplaceProvider;

impl Provider for ArrayReplaceProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ArrayReplaceProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact(b"array_replace")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let mut replaced: Option<TKeyedArray> = None;
        for argument in invocation.arguments() {
            let argument_type = context.get_expression_type(argument.value()?)?;
            if argument.is_unpacked() || !argument_type.is_single() {
                return None;
            }

            let TAtomic::Array(TArray::Keyed(keyed)) = argument_type.get_single() else {
                return None;
            };

            replaced = Some(match replaced {
                Some(replaced) => compose_keyed_plus(keyed, &replaced),
                None => keyed.clone(),
            });
        }

        Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(replaced?))))
    }
}
