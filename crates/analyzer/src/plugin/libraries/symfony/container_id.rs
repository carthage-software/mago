//! String-id `ContainerInterface::get('service.id')` return type provider.
//!
//! STATUS: WIP scaffold. Design:
//!
//! Configuration: an analyzer setting (e.g. `symfony.container-xml-path`)
//! pointing at the compiled container dump — for a Symfony app under test,
//! `var/cache/test/App_KernelTestDebugContainer.xml`. Parsed once at plugin
//! construction into an id -> class map (plus aliases).
//!
//! Provider: same shape as `psr_container::ContainerGetProvider`, but where
//! that provider requires a class-string argument, this one accepts a literal
//! string id and resolves it through the map. Falls back to `None` (no
//! opinion) when the id is unknown or the argument is not a literal.
//!
//! This mirrors what phpstan-symfony's `containerXmlPath` provides, measured
//! on a production codebase: without it, every string-id fetch is `mixed`.

/// Provider resolving string service ids through the compiled container XML.
#[derive(Default)]
pub struct ContainerIdProvider;
