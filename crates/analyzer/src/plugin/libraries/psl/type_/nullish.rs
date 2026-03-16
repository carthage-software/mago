//! `Psl\Type\nullish()` return type provider.

use mago_atom::atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("psl::type::nullish", "Psl\\Type\\nullish", "Returns TypeInterface with nullable inner type");

/// Provider for the `Psl\Type\nullish()` function.
///
/// Returns a `TypeInterface` with the inner type marked as nullable.
#[derive(Default)]
pub struct NullishProvider;

impl Provider for NullishProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for NullishProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("psl\\type\\nullish")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let type_interface = invocation.get_argument(0, &["inner_type"])?;
        let type_interface_type = context.get_expression_type(type_interface)?;

        let inner_type = type_interface_type
            .get_single_named_object()?
            .get_type_parameters()
            .and_then(|type_parameters| type_parameters.first())
            .cloned()?
            .as_nullable();

        Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            atom("Psl\\Type\\TypeInterface"),
            Some(vec![inner_type]),
        )))))
    }
}
