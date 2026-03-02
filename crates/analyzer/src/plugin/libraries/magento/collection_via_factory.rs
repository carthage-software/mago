//! Rule: Collections should be retrieved via factory, not via model.
//!
//! Calling `getCollection()` on objects extending `Magento\Framework\Model\AbstractModel`
//! is deprecated since Magento 101.0.0. Collections should be instantiated directly
//! via their factory class instead.

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::MethodCall;

use crate::code::IssueCode;
use crate::plugin::context::HookContext;
use crate::plugin::hook::HookResult;
use crate::plugin::MethodCallHook;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;

use super::utils::get_method_name;
use super::utils::get_receiver_class_names;

static META: ProviderMeta = ProviderMeta::new(
    "magento::collection-via-factory",
    "Collection Via Factory",
    "Flags getCollection() calls on AbstractModel",
);

#[derive(Default)]
pub struct CollectionViaFactoryHook;

impl Provider for CollectionViaFactoryHook {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodCallHook for CollectionViaFactoryHook {
    fn after_method_call(&self, call: &MethodCall<'_>, context: &mut HookContext<'_, '_>) -> HookResult<()> {
        let Some(method_name) = get_method_name(call) else {
            return Ok(());
        };

        if !method_name.eq_ignore_ascii_case("getCollection") {
            return Ok(());
        }

        for class_name in get_receiver_class_names(call, context) {
            if context.is_instance_of(&class_name, "Magento\\Framework\\Model\\AbstractModel") {
                context.report(
                    IssueCode::MagentoCollectionViaFactory,
                    Issue::warning(format!(
                        "Collections should be retrieved via factory, not via `{}::getCollection()`.",
                        class_name,
                    ))
                    .with_annotation(
                        Annotation::primary(call.span())
                            .with_message("Use the collection factory instead"),
                    )
                    .with_help(
                        "Inject the collection factory (e.g. `CollectionFactory`) via constructor \
                         and use `$this->collectionFactory->create()` instead.",
                    ),
                );

                break;
            }
        }

        Ok(())
    }
}
