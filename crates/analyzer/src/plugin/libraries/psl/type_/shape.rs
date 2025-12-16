//! `Psl\Type\shape()` return type provider.

use std::collections::BTreeMap;

use mago_atom::atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "psl::type::shape",
    "Psl\\Type\\shape",
    "Returns TypeInterface with array shape based on element types",
);

/// Provider for the `Psl\Type\shape()` function.
///
/// Returns a `TypeInterface` with the array shape inferred from the input types.
#[derive(Default)]
pub struct ShapeProvider;

impl Provider for ShapeProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ShapeProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("psl\\type\\shape")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let elements = invocation.get_argument(0, &["elements"])?;
        let elements_type = context.get_expression_type(elements)?;

        let argument_array = if let Some(argument_array) = elements_type.get_single_array()
            && argument_array.is_sealed()
        {
            argument_array
        } else {
            return None;
        };

        let allows_unknown_elements = if let Some(argument) = invocation.get_argument(1, &["allow_unknown_fields"]) {
            context
                .get_expression_type(argument)
                .and_then(|union| union.get_single_bool())
                .filter(|boolean| !boolean.is_general())
                .map(mago_codex::ttype::atomic::scalar::bool::TBool::is_true)?
        } else {
            false // default to false if not provided
        };

        match argument_array {
            TArray::List(list) => {
                let mut known_elements = BTreeMap::new();
                for (index, (possibly_undefined, element)) in list.known_elements.as_ref()? {
                    let inner_type = element
                        .get_single_named_object()?
                        .type_parameters
                        .as_ref()
                        .and_then(|type_parameters| type_parameters.first())
                        .cloned()?;

                    let possibly_undefined = *possibly_undefined || element.possibly_undefined();

                    known_elements.insert(*index, (possibly_undefined, inner_type));
                }

                Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                    atom("Psl\\Type\\TypeInterface"),
                    Some(vec![TUnion::from_atomic(TAtomic::Array(TArray::List(TList {
                        element_type: if allows_unknown_elements {
                            Box::new(get_mixed())
                        } else {
                            Box::new(get_never())
                        },
                        known_count: Some(known_elements.len()),
                        non_empty: !known_elements.is_empty(),
                        known_elements: Some(known_elements),
                    })))]),
                )))))
            }
            TArray::Keyed(keyed_array) => {
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

                Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                    atom("Psl\\Type\\TypeInterface"),
                    Some(vec![TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                        parameters: if allows_unknown_elements {
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
    }
}
