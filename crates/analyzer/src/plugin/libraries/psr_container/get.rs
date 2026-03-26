//! `Psr\Container\ContainerInterface::get()` return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;

static META: ProviderMeta = ProviderMeta::new(
    "psr::container::get",
    "ContainerInterface::get",
    "Returns the object type matching the class-string argument",
);

// Use wildcard for class since many classes implement ContainerInterface
static TARGETS: [MethodTarget; 1] = [MethodTarget::any_class("get")];

/// Provider for the `Psr\Container\ContainerInterface::get()` method.
///
/// When called with a class-string argument like `SomeService::class`,
/// returns the object type of that class instead of `mixed`.
#[derive(Default)]
pub struct ContainerGetProvider;

impl Provider for ContainerGetProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodReturnTypeProvider for ContainerGetProvider {
    fn targets() -> &'static [MethodTarget] {
        &TARGETS
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        class_name: &str,
        _method_name: &str,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        // Only handle classes that implement ContainerInterface
        if !context.is_instance_of(class_name, "Psr\\Container\\ContainerInterface") {
            return None;
        }

        // Get the first argument (the service ID)
        let id_arg = invocation.get_argument(0, &["id"])?;
        let id_type = context.get_expression_type(id_arg)?;

        // Extract class-string value (handles SomeClass::class)
        let service_class = id_type.get_single_class_string_value()?;

        // Return object type of that class
        Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(service_class)))))
    }
}
