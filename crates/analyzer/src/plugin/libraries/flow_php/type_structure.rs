//! `Flow\Types\DSL\type_structure()` return type provider.

use std::collections::BTreeMap;

use mago_atom::atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "flow_php::types::type_structure",
    "Flow\\Types\\DSL\\type_structure",
    "Returns Flow\\Types\\Type with structure shape",
);

/// Provider for the `Flow\Types\DSL\type_structure()` function.
///
/// Returns a Type with the structure shape inferred from the input types.
#[derive(Default)]
pub struct TypeStructureProvider;

impl Provider for TypeStructureProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for TypeStructureProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("flow\\types\\dsl\\type_structure")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let elements = invocation.get_argument(0, &["elements"])?;
        let elements_type = context.get_expression_type(elements)?;

        let elements_array = if let Some(elements_array) = elements_type.get_single_array()
            && elements_array.is_sealed()
        {
            elements_array
        } else {
            return None;
        };

        let optional_elements_array =
            if let Some(optional_elements) = invocation.get_argument(1, &["optional_elements"]) {
                let optional_elements_type = context.get_expression_type(optional_elements)?;
                if let Some(optional_array) = optional_elements_type.get_single_array()
                    && optional_array.is_sealed()
                {
                    Some(optional_array)
                } else {
                    None
                }
            } else {
                None
            };

        let allows_extra_fields = if let Some(argument) = invocation.get_argument(2, &["allow_extra"]) {
            context
                .get_expression_type(argument)
                .and_then(|union| union.get_single_bool())
                .filter(|boolean| !boolean.is_general())
                .map(mago_codex::ttype::atomic::scalar::bool::TBool::is_true)?
        } else {
            false
        };

        let keyed_array = if let TArray::Keyed(keyed_array) = elements_array {
            keyed_array
        } else {
            return None;
        };

        let mut known_items = BTreeMap::new();
        for (key, (possibly_undefined, item)) in keyed_array.known_items.as_ref()? {
            let inner_type = item
                .get_single_named_object()?
                .type_parameters
                .as_ref()
                .and_then(|type_parameters| type_parameters.first())
                .cloned()?;

            let possibly_undefined = *possibly_undefined || inner_type.possibly_undefined();

            known_items.insert(*key, (possibly_undefined, inner_type));
        }

        if let Some(TArray::Keyed(optional_keyed_array)) = optional_elements_array
            && let Some(optional_items) = optional_keyed_array.known_items.as_ref()
        {
            for (key, (_possibly_undefined, item)) in optional_items {
                let inner_type = item
                    .get_single_named_object()?
                    .type_parameters
                    .as_ref()
                    .and_then(|type_parameters| type_parameters.first())
                    .cloned()?;

                known_items.insert(*key, (true, inner_type));
            }
        }

        Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            atom("Flow\\Types\\Type"),
            Some(vec![TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                parameters: if allows_extra_fields {
                    Some((Box::new(get_arraykey()), Box::new(get_mixed())))
                } else {
                    None
                },
                non_empty: !known_items.is_empty(),
                known_items: Some(known_items),
            })))]),
        )))))
    }
}
