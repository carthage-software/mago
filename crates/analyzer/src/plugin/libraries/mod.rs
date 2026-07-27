//! Library-specific providers for PHP stdlib, PSL, Flow-PHP, PSR-11 Container, Doctrine ORM, and Symfony.

pub mod doctrine;
pub mod flow_php;
pub mod psl;
pub mod psr_container;
pub mod stdlib;
pub mod symfony;

use crate::plugin::Plugin;

pub use doctrine::DoctrinePlugin;
pub use flow_php::FlowPhpPlugin;
pub use psl::PslPlugin;
pub use psr_container::PsrContainerPlugin;
pub use stdlib::StdlibPlugin;
pub use symfony::SymfonyPlugin;

/// All available analyzer plugins.
pub static ALL_PLUGINS: &[&dyn Plugin] =
    &[&StdlibPlugin, &PslPlugin, &FlowPhpPlugin, &PsrContainerPlugin, &DoctrinePlugin, &SymfonyPlugin];
