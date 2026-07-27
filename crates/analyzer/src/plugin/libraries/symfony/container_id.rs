//! String-id `ContainerInterface::get('service.id')` return type provider.
//!
//! Same shape as `psr_container::ContainerGetProvider`, but where that
//! provider requires a class-string argument, this one accepts a literal
//! string id and resolves it through the service map parsed from the
//! compiled container XML. Falls back to `None` (no opinion) when the id is
//! unknown or the argument is not a literal: fail-open on typing, never a
//! false positive.
//!
//! This mirrors what phpstan-symfony's `containerXmlPath` provides: without
//! it, every string-id fetch is `mixed`.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::libraries::symfony::ServiceMap;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;

static META: ProviderMeta = ProviderMeta::new(
    "symfony::container::get",
    "ContainerInterface::get with a string id",
    "Returns the object type of the service the compiled container maps the string id to",
);

// Use wildcard for the class since many classes implement ContainerInterface
static TARGETS: [MethodTarget; 1] = [MethodTarget::any_class(b"get")];

/// Provider resolving literal string service ids through the compiled
/// container XML.
///
/// When called with a literal string id like `'app.mailer'`, returns the
/// object type of the class the container maps that id to (following
/// aliases). Unknown ids and non-literal arguments produce no opinion.
pub struct ContainerIdProvider {
    services: ServiceMap,
}

impl ContainerIdProvider {
    /// Creates the provider from an already-parsed service map.
    #[must_use]
    pub const fn new(services: ServiceMap) -> Self {
        Self { services }
    }
}

impl Provider for ContainerIdProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodReturnTypeProvider for ContainerIdProvider {
    fn targets() -> &'static [MethodTarget] {
        &TARGETS
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        class_name: &[u8],
        _method_name: &[u8],
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        if !context.is_instance_of(class_name, b"Psr\\Container\\ContainerInterface") {
            return None;
        }

        let id_arg = invocation.get_argument(0, &[b"id"])?;
        let id_type = context.get_expression_type(id_arg)?;

        // Extract the literal string id (e.g. `'app.mailer'`)
        let service_id = id_type.get_single_literal_string_value()?;

        // Resolve it through the compiled container's service map
        let service_class = self.services.class_of(service_id)?;

        // Return object type of that class
        Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(service_class)))))
    }
}
