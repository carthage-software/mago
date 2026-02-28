//! Library-specific providers for PHP stdlib, PSL, Flow-PHP, PSR-11 Container, and Magento 2.

pub mod flow_php;
pub mod magento;
pub mod psl;
pub mod psr_container;
pub mod stdlib;

use crate::plugin::Plugin;

pub use flow_php::FlowPhpPlugin;
pub use magento::MagentoPlugin;
pub use psl::PslPlugin;
pub use psr_container::PsrContainerPlugin;
pub use stdlib::StdlibPlugin;

/// All available analyzer plugins.
pub static ALL_PLUGINS: &[&dyn Plugin] =
    &[&StdlibPlugin, &PslPlugin, &FlowPhpPlugin, &PsrContainerPlugin, &MagentoPlugin];
