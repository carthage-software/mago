//! Library-specific providers for PHP stdlib, PSL, Flow-PHP, and PSR-11 Container.

pub mod flow_php;
pub mod psl;
pub mod psr_container;
pub mod stdlib;

use crate::plugin::Plugin;

pub use flow_php::FlowPhpPlugin;
pub use psl::PslPlugin;
pub use psr_container::PsrContainerPlugin;
pub use stdlib::StdlibPlugin;

/// All available analyzer plugins.
pub static ALL_PLUGINS: &[&dyn Plugin] = &[&StdlibPlugin, &PslPlugin, &FlowPhpPlugin, &PsrContainerPlugin];
