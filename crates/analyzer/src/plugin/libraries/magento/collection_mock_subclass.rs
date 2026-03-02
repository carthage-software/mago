//! Rule: `getCollectionMock()` requires a Collection subclass.
//!
//! When using `Magento\Framework\TestFramework\Unit\Helper\ObjectManager::getCollectionMock()`,
//! the first argument must be a class name that extends `Magento\Framework\Data\Collection`.

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
    "magento::collection-mock-subclass",
    "Collection Mock Subclass",
    "Validates getCollectionMock() argument extends Collection",
);

#[derive(Default)]
pub struct CollectionMockSubclassHook;

impl Provider for CollectionMockSubclassHook {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodCallHook for CollectionMockSubclassHook {
    fn after_method_call(&self, call: &MethodCall<'_>, context: &mut HookContext<'_, '_>) -> HookResult<()> {
        let Some(method_name) = get_method_name(call) else {
            return Ok(());
        };

        if !method_name.eq_ignore_ascii_case("getCollectionMock") {
            return Ok(());
        }

        // Check if the receiver is the Magento test ObjectManager
        let is_object_manager = get_receiver_class_names(call, context).into_iter().any(|class_name| {
            context.is_instance_of(
                &class_name,
                "Magento\\Framework\\TestFramework\\Unit\\Helper\\ObjectManager",
            )
        });

        if !is_object_manager {
            return Ok(());
        }

        // Get the first argument (the class name string)
        let Some(first_arg) = call.argument_list.arguments.first() else {
            return Ok(());
        };

        let arg_expr = first_arg.value();
        let Some(arg_type) = context.get_expression_type(arg_expr) else {
            return Ok(());
        };

        // Extract the class name from the class-string argument
        let Some(class_string_value) = arg_type.get_single_class_string_value() else {
            return Ok(());
        };

        // Check if the class extends Magento\Framework\Data\Collection
        if !context.is_instance_of(class_string_value.as_str(), "Magento\\Framework\\Data\\Collection") {
            context.report(
                IssueCode::MagentoCollectionMockSubclass,
                Issue::error(format!(
                    "`{}` does not extend `\\Magento\\Framework\\Data\\Collection` as required by `getCollectionMock()`.",
                    class_string_value,
                ))
                .with_annotation(
                    Annotation::primary(arg_expr.span())
                        .with_message("This class must extend Magento\\Framework\\Data\\Collection"),
                )
                .with_help("Pass a class name that extends \\Magento\\Framework\\Data\\Collection."),
            );
        }

        Ok(())
    }
}
