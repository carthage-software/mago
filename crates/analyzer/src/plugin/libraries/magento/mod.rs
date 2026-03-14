//! Magento 2 specific providers and hooks.
//!
//! This plugin provides type-aware checks for Magento 2 projects, based on the rules
//! from phpstan-magento. It includes:
//!
//! - Architectural rules enforcing Magento best practices
//! - ObjectManager return type resolution

mod collection_mock_subclass;
mod collection_via_factory;
mod no_set_template_in_block;
mod object_manager;
mod use_resource_model_directly;
mod use_service_contracts;
mod utils;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;

pub use collection_mock_subclass::CollectionMockSubclassHook;
pub use collection_via_factory::CollectionViaFactoryHook;
pub use no_set_template_in_block::NoSetTemplateInBlockHook;
pub use object_manager::ObjectManagerReturnTypeProvider;
pub use use_resource_model_directly::UseResourceModelDirectlyHook;
pub use use_service_contracts::UseServiceContractsHook;

pub struct MagentoPlugin;

static META: PluginMeta = PluginMeta::new(
    "magento",
    "Magento 2",
    "Type providers and architectural rules for Magento 2 projects",
    &["magento2", "mage-os"],
    false,
);

impl Plugin for MagentoPlugin {
    fn meta(&self) -> &'static PluginMeta {
        &META
    }

    fn register(&self, registry: &mut PluginRegistry) {
        // Type providers
        registry.register_method_provider(ObjectManagerReturnTypeProvider);

        // Architectural rules (method call hooks)
        registry.register_method_call_hook(NoSetTemplateInBlockHook);
        registry.register_method_call_hook(UseServiceContractsHook);
        registry.register_method_call_hook(CollectionViaFactoryHook);
        registry.register_method_call_hook(UseResourceModelDirectlyHook);
        registry.register_method_call_hook(CollectionMockSubclassHook);
    }
}
