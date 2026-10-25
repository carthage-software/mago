//! `Closure::fromCallable()` return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::cast::cast_atomic_to_callable;
use mago_codex::ttype::expander::get_parameter_dependent_signature_of_function_like_identifier;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::closure::fromcallable",
    "Closure::fromCallable",
    "Preserves the converted callable's signature",
);

static TARGETS: [MethodTarget; 1] = [MethodTarget::exact(b"closure", b"fromcallable")];

#[derive(Default)]
pub struct ClosureFromCallableProvider;

impl Provider for ClosureFromCallableProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodReturnTypeProvider for ClosureFromCallableProvider {
    fn targets() -> &'static [MethodTarget] {
        &TARGETS
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        _class_name: &[u8],
        _method_name: &[u8],
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let callback = invocation.get_argument(0, &[b"callback"])?;
        let callback_type = context.get_expression_type(callback)?;
        let codebase = context.codebase();
        let mut closures = Vec::with_capacity(callback_type.types.len());

        for atomic in callback_type.types.as_ref() {
            // Fall back to the declared Closure type if any alternative cannot be resolved.
            let callable = cast_atomic_to_callable(atomic, codebase, None)?;
            let mut signature = match callable.as_ref() {
                TCallable::Signature(signature) => signature.clone(),
                TCallable::Alias(identifier) => {
                    get_parameter_dependent_signature_of_function_like_identifier(identifier, codebase)?
                }
            };

            signature.is_closure = true;
            closures.push(TAtomic::Callable(TCallable::Signature(signature)));
        }

        Some(TUnion::from_vec(closures))
    }
}
