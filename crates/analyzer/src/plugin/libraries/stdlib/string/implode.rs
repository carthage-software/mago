//! `implode()` return type provider.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::union::TUnion;

use crate::expression::binary::concat::get_operand_strings;
use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::string::implode",
    "implode",
    "Returns a non-empty string when a piece, or the separator between pieces, is never empty",
);

#[derive(Default)]
pub struct ImplodeProvider;

impl Provider for ImplodeProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ImplodeProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact(b"implode")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let (separator_type, pieces_type) = match invocation.get_argument(1, &[b"array"]) {
            Some(pieces) => (
                Some(context.get_expression_type(invocation.get_argument(0, &[b"separator"])?)?),
                context.get_expression_type(pieces)?,
            ),
            None => (None, context.get_expression_type(invocation.get_argument(0, &[b"separator"])?)?),
        };

        let TAtomic::Array(array) = (pieces_type.is_single().then(|| pieces_type.get_single()))? else {
            return None;
        };

        let required_pieces: Vec<&TUnion> = match array {
            TArray::List(list) => list
                .known_elements
                .as_ref()?
                .values()
                .filter(|(optional, _)| !optional)
                .map(|(_, piece)| piece)
                .collect(),
            TArray::Keyed(keyed) => {
                keyed.known_items.as_ref()?.values().filter(|(optional, _)| !optional).map(|(_, piece)| piece).collect()
            }
        };

        let is_non_empty = |piece: &TUnion| get_operand_strings(piece).iter().all(|string| string.is_non_empty());
        let all_non_empty = required_pieces.iter().all(|piece| is_non_empty(piece));
        let any_non_empty = required_pieces.iter().any(|piece| is_non_empty(piece));
        let separator_non_empty = separator_type.is_some_and(|separator| {
            separator.types.iter().all(|atomic| {
                matches!(atomic, TAtomic::Scalar(_))
                    && get_operand_strings(separator).iter().all(|string| string.is_non_empty())
            })
        });

        (!all_non_empty && (any_non_empty || (separator_non_empty && required_pieces.len() >= 2)))
            .then(get_non_empty_string)
    }
}
