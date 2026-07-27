//! Configuration consumed by plugins at registration time.

use std::path::PathBuf;

/// Settings passed to every [`Plugin::register`](crate::plugin::Plugin::register) call.
///
/// Most plugins are configuration-free and ignore these settings entirely.
/// Fields are namespaced by plugin id (e.g. `symfony_*` for the `symfony`
/// plugin), so a single flat struct stays unambiguous as plugins grow.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PluginSettings {
    /// Path to a compiled Symfony container XML dump, used by the `symfony`
    /// plugin to resolve string service ids to class names.
    ///
    /// For an application under test this is typically
    /// `var/cache/test/App_KernelTestDebugContainer.xml` — the same file
    /// phpstan-symfony consumes through its `containerXmlPath` parameter.
    ///
    /// When `None`, the `symfony` plugin registers no container-id provider.
    pub symfony_container_xml_path: Option<PathBuf>,
}
