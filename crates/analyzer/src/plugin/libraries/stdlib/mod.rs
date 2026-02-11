//! PHP standard library providers.

pub mod array;
pub mod closure;
pub mod r#enum;
pub mod filter;
pub mod json;
pub mod math;
pub mod random;
pub mod spl;
pub mod string;
pub mod url;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;

/// Plugin providing type inference for PHP standard library functions.
pub struct StdlibPlugin;

static META: PluginMeta = PluginMeta::new(
    "stdlib",
    "PHP Standard Library",
    "Type providers for PHP built-in functions (strlen, array_*, json_*, etc.)",
    &["standard", "std", "php-stdlib"],
    true,
);

impl Plugin for StdlibPlugin {
    fn meta(&self) -> &'static PluginMeta {
        &META
    }

    fn register(&self, registry: &mut PluginRegistry) {
        registry.register_function_provider(string::StrlenProvider);
        registry.register_function_provider(json::JsonEncodeProvider);
        registry.register_function_provider(random::RandProvider);
        registry.register_function_provider(random::RandomIntProvider);
        registry.register_function_provider(random::RandomBytesProvider);
        registry.register_function_provider(spl::IteratorToArrayProvider);
        registry.register_function_provider(array::ArrayColumnProvider);
        registry.register_function_provider(array::ArrayFilterProvider);
        registry.register_function_provider(array::ArrayMergeProvider);
        registry.register_function_provider(array::CompactProvider);
        registry.register_function_provider(url::ParseUrlProvider);
        registry.register_function_provider(filter::FilterVarProvider);
        registry.register_function_provider(filter::FilterInputProvider);
        registry.register_function_provider(math::MinProvider);
        registry.register_function_provider(math::MaxProvider);

        registry.register_method_provider(closure::ClosureGetCurrentProvider);
        registry.register_method_provider(r#enum::EnumCasesProvider);
    }
}
