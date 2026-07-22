//! `array_is_list()` return type provider.
//!
//! A generic `array<Tk, Tv>` with `Tk of array-key` may be instantiated as a
//! list, but its conditional stub return type cannot preserve that dependency.
//! Keep the return as `bool` in that case; the assertion reconciler still
//! narrows the true branch to `list<Tv>`.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::get_bool;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::array::array_is_list",
    "array_is_list",
    "Keeps generic array key/list checks indeterminate until narrowed",
);

#[derive(Default)]
pub struct ArrayIsListProvider;

impl Provider for ArrayIsListProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ArrayIsListProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact(b"array_is_list")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let array = invocation.get_argument(0, &[b"array"])?;
        let array_type = context.get_expression_type(array)?;

        array_type.types.iter().any(generic_keyed_array_can_be_list).then_some(get_bool())
    }
}

fn generic_keyed_array_can_be_list(atomic: &TAtomic) -> bool {
    let TAtomic::Array(TArray::Keyed(keyed)) = atomic else {
        return false;
    };

    let Some((key_type, _)) = keyed.parameters.as_ref() else {
        return false;
    };

    key_type.types.iter().any(
        |key_atomic| matches!(key_atomic, TAtomic::GenericParameter(parameter) if parameter.constraint.is_array_key()),
    )
}
