//! Rule: Use service contracts to persist entities.
//!
//! Direct calls to `save()`, `load()`, and `delete()` on objects extending
//! `Magento\Framework\Model\AbstractModel` are deprecated since Magento 100.1.0.
//! Use repository service contracts instead.

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

const DEPRECATED_METHODS: &[&str] = &["save", "load", "delete"];

static META: ProviderMeta = ProviderMeta::new(
    "magento::use-service-contracts",
    "Use Service Contracts",
    "Flags direct save/load/delete calls on AbstractModel",
);

#[derive(Default)]
pub struct UseServiceContractsHook;

impl Provider for UseServiceContractsHook {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodCallHook for UseServiceContractsHook {
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
                    IssueCode::MagentoUseServiceContracts,
                    Issue::warning(format!(
                        "Use service contracts to persist entities instead of `{}::{}()`.",
                        class_name, matched,
                    ))
                    .with_annotation(
                        Annotation::primary(call.span())
                            .with_message("Direct model persistence is deprecated"),
                    )
                    .with_help(
                        "Use the corresponding repository interface (e.g. `ProductRepositoryInterface::save()`) instead.",
                    ),
                );

                break;
            }
        }

        Ok(())
    }
}
