//! Rule: Do not use `setTemplate()` in Block classes.
//!
//! Setter methods like `setTemplate()` are discouraged in classes extending
//! `Magento\Framework\View\Element\Template`. Templates should be set via
//! layout XML or constructor arguments instead.
//!
//! See: <https://github.com/extdn/extdn-phpcs/blob/master/Extdn/Sniffs/Blocks/SetTemplateInBlockSniff.md>

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
    "magento::no-set-template-in-block",
    "No setTemplate in Block",
    "Flags setTemplate() calls on Block classes",
);

#[derive(Default)]
pub struct NoSetTemplateInBlockHook;

impl Provider for NoSetTemplateInBlockHook {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodCallHook for NoSetTemplateInBlockHook {
    fn after_method_call(&self, call: &MethodCall<'_>, context: &mut HookContext<'_, '_>) -> HookResult<()> {
        let Some(method_name) = get_method_name(call) else {
            return Ok(());
        };

        if !method_name.eq_ignore_ascii_case("setTemplate") {
            return Ok(());
        }

        for class_name in get_receiver_class_names(call, context) {
            if context.is_instance_of(&class_name, "Magento\\Framework\\View\\Element\\Template") {
                context.report(
                    IssueCode::MagentoNoSetTemplateInBlock,
                    Issue::warning(format!(
                        "Calling `{}::setTemplate()` is discouraged in Block classes.",
                        class_name
                    ))
                    .with_annotation(
                        Annotation::primary(call.span())
                            .with_message("Use layout XML or constructor arguments to set the template instead"),
                    )
                    .with_help(
                        "Set the template in layout XML using the `template` attribute, \
                         or pass it as a constructor argument.",
                    ),
                );

                break;
            }
        }

        Ok(())
    }
}
