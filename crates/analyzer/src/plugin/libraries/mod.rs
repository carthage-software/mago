//! Library-specific providers for PHP stdlib, PSL, and Flow-PHP.

pub mod flow_php;
pub mod psl;
pub mod stdlib;

use crate::plugin::PluginRegistry;

pub fn register_library_providers(registry: &mut PluginRegistry) {
    registry.register_function_provider(stdlib::string::StrlenProvider);
    registry.register_function_provider(stdlib::json::JsonEncodeProvider);
    registry.register_function_provider(stdlib::random::RandProvider);
    registry.register_function_provider(stdlib::random::RandomIntProvider);
    registry.register_function_provider(stdlib::random::RandomBytesProvider);
    registry.register_function_provider(stdlib::spl::IteratorToArrayProvider);
    registry.register_function_provider(stdlib::array::ArrayColumnProvider);
    registry.register_function_provider(stdlib::array::ArrayFilterProvider);
    registry.register_function_provider(stdlib::array::ArrayMergeProvider);
    registry.register_function_provider(stdlib::array::CompactProvider);
    registry.register_method_provider(stdlib::closure::ClosureGetCurrentProvider);
    registry.register_method_provider(stdlib::r#enum::EnumCasesProvider);
    registry.register_function_provider(stdlib::url::ParseUrlProvider);

    registry.register_function_provider(psl::type_::ShapeProvider);
    registry.register_function_provider(psl::type_::OptionalProvider);
    registry.register_function_provider(psl::str::StrProvider);
    registry.register_function_provider(psl::regex::CaptureGroupsProvider);

    registry.register_function_provider(flow_php::TypeStructureProvider);
}
