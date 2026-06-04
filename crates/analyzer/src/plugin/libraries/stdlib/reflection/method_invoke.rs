//! `ReflectionMethod<T>::invoke()` / `invokeArgs()` return type provider.

use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;

static META: ProviderMeta = ProviderMeta::new(
    "reflection::method::invoke",
    "ReflectionMethod::invoke",
    "Returns the union of method return types of the reflected class",
);

static TARGETS: [MethodTarget; 2] = [MethodTarget::any_class(b"invoke"), MethodTarget::any_class(b"invokeArgs")];

/// Provider for `ReflectionMethod<T>::invoke()` and `invokeArgs()`.
///
/// A `ReflectionMethod<T>` does not track *which* method it is, so invoking it
/// returns the union of the return types of every method on `T` (e.g. invoking
/// some method of a class whose only method returns `'Hello'` yields `'Hello'`)
/// instead of `mixed`.
#[derive(Default)]
pub struct ReflectionMethodInvokeProvider;

impl Provider for ReflectionMethodInvokeProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodReturnTypeProvider for ReflectionMethodInvokeProvider {
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
        let reflected_class = super::reflected_class_name(context, invocation, b"ReflectionMethod")?;

        super::method_return_union(context.codebase(), reflected_class)
    }
}
