//! `ReflectionMethod<T>::getName()` return type provider.

use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;

static META: ProviderMeta = ProviderMeta::new(
    "reflection::method::get_name",
    "ReflectionMethod::getName",
    "Returns the union of method names of the reflected class",
);

static TARGETS: [MethodTarget; 1] = [MethodTarget::any_class(b"getName")];

/// Provider for `ReflectionMethod<T>::getName()`.
///
/// A `ReflectionMethod<T>` obtained from `getMethods()`/`getMethod()` does not
/// track *which* method it is, so `getName()` returns the union of every method
/// name on `T` (e.g. `'greet'|'count'`) instead of a plain `string`. This lets
/// dynamic dispatch like `$object->{$method->getName()}()` resolve.
#[derive(Default)]
pub struct ReflectionMethodGetNameProvider;

impl Provider for ReflectionMethodGetNameProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodReturnTypeProvider for ReflectionMethodGetNameProvider {
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

        super::method_name_union(context.codebase(), reflected_class)
    }
}
