//! `ReflectionClass<T>::hasMethod()` assertion provider.

use mago_codex::assertion::Assertion;
use mago_word::word;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::assertion::InvocationAssertions;
use crate::plugin::provider::assertion::MethodAssertionProvider;
use crate::plugin::provider::method::MethodTarget;

static META: ProviderMeta = ProviderMeta::new(
    "reflection::class::has_method",
    "ReflectionClass::hasMethod",
    "Narrows the argument to a method name of the reflected class when true",
);

static TARGETS: [MethodTarget; 1] = [MethodTarget::any_class(b"hasMethod")];

/// Provider for `ReflectionClass<T>::hasMethod($name)`.
///
/// When the call returns true, `$name` is one of `T`'s method names, so it is
/// narrowed to the union of `T`'s method-name literals (e.g. `'greet'|'count'`)
/// inside the truthy branch. This lets `if ($reflection->hasMethod($name)) {
/// $object->{$name}(); }` resolve the dynamic call.
#[derive(Default)]
pub struct ReflectionClassHasMethodAssertionProvider;

impl Provider for ReflectionClassHasMethodAssertionProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodAssertionProvider for ReflectionClassHasMethodAssertionProvider {
    fn targets() -> &'static [MethodTarget] {
        &TARGETS
    }

    fn get_assertions(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        _class_name: &[u8],
        _method_name: &[u8],
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<InvocationAssertions> {
        let reflected_class = super::reflected_class_name(context, invocation, b"ReflectionClass")?;
        let method_names = super::method_name_union(context.codebase(), reflected_class)?;

        let mut assertions = InvocationAssertions::new();
        assertions.add_if_true(word("$name"), vec![Assertion::InArray(method_names)]);

        Some(assertions)
    }
}
