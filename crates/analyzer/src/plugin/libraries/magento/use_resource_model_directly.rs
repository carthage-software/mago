//! Rule: Resource models should be used directly.
//!
//! Calling `getResource()` or `_getResource()` on objects extending
//! `Magento\Framework\Model\AbstractModel` is deprecated since Magento 100.1.0.
//! Resource models should be injected directly via constructor DI.

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

const DEPRECATED_METHODS: &[&str] = &["getResource", "_getResource"];

static META: ProviderMeta = ProviderMeta::new(
    "magento::use-resource-model-directly",
    "Use Resource Model Directly",
    "Flags getResource()/_getResource() calls on AbstractModel",
);

#[derive(Default)]
pub struct UseResourceModelDirectlyHook;

impl Provider for UseResourceModelDirectlyHook {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodCallHook for UseResourceModelDirectlyHook {
    fn after_method_call(&self, call: &MethodCall<'_>, context: &mut HookContext<'_, '_>) -> HookResult<()> {
        let Some(method_name) = get_method_name(call) else {
            return Ok(());
        };

        let matched_method = DEPRECATED_METHODS
            .iter()
            .find(|&&m| method_name.eq_ignore_ascii_case(m));

        let Some(&matched) = matched_method else {
            return Ok(());
        };

        for class_name in get_receiver_class_names(call, context) {
            if context.is_instance_of(&class_name, "Magento\\Framework\\Model\\AbstractModel") {
                context.report(
                    IssueCode::MagentoUseResourceModelDirectly,
                    Issue::warning(format!(
                        "`{}::{}()` is deprecated. Use resource models directly.",
                        class_name, matched,
                    ))
                    .with_annotation(
                        Annotation::primary(call.span())
                            .with_message("Inject the resource model via constructor instead"),
                    )
                    .with_help(
                        "Inject the resource model directly via constructor DI \
                         and call its methods instead.",
                    ),
                );

                break;
            }
        }

        Ok(())
    }
}
