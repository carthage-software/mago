//! Return type provider for `Magento\Framework\ObjectManagerInterface::get()` and `::create()`.
//!
//! Resolves the return type based on the class-string argument, so that
//! `$objectManager->get(Foo::class)` returns `Foo` and
//! `$objectManager->create(Bar::class)` returns `Bar`.

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
    "magento::object-manager",
    "ObjectManager::get/create",
    "Returns the object type matching the class-string argument for ObjectManager",
);

static TARGETS: [MethodTarget; 2] = [
    MethodTarget::any_class("get"),
    MethodTarget::any_class("create"),
];

#[derive(Default)]
pub struct ObjectManagerReturnTypeProvider;

impl Provider for ObjectManagerReturnTypeProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodReturnTypeProvider for ObjectManagerReturnTypeProvider {
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
        // Only handle classes that implement ObjectManagerInterface
        if !context.is_instance_of(class_name, "Magento\\Framework\\ObjectManagerInterface") {
            return None;
        }

        // Get the first argument (the class/type name)
        let type_arg = invocation.get_argument(0, &["type", "id"])?;
        let type_type = context.get_expression_type(type_arg)?;

        // Extract the class-string value (handles Foo::class expressions)
        let requested_class = type_type.get_single_class_string_value()?;

        Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(
            TNamedObject::new(requested_class),
        ))))
    }
}
