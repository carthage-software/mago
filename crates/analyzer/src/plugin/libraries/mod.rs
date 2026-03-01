//! Library-specific providers for PHP stdlib, PSL, Flow-PHP, PSR-11 Container, and Laravel.

pub mod flow_php;
pub mod laravel;
pub mod psl;
pub mod psr_container;
pub mod stdlib;

use crate::plugin::Plugin;

pub use flow_php::FlowPhpPlugin;
pub use laravel::LaravelPlugin;
pub use psl::PslPlugin;
pub use psr_container::PsrContainerPlugin;
pub use stdlib::StdlibPlugin;

/// All available analyzer plugins.
pub static ALL_PLUGINS: &[&dyn Plugin] = &[&StdlibPlugin, &PslPlugin, &FlowPhpPlugin, &PsrContainerPlugin, &LaravelPlugin];