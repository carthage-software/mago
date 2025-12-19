//! Library-specific providers for PHP stdlib, PSL, and Flow-PHP.

pub mod flow_php;
pub mod psl;
pub mod stdlib;

use crate::plugin::Plugin;

pub use flow_php::FlowPhpPlugin;
pub use psl::PslPlugin;
pub use stdlib::StdlibPlugin;

/// All available analyzer plugins.
pub static ALL_PLUGINS: &[&dyn Plugin] = &[&StdlibPlugin, &PslPlugin, &FlowPhpPlugin];
