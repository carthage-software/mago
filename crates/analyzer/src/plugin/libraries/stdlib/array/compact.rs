//! `compact()` return type provider.

use std::collections::BTreeMap;
use std::sync::Arc;

use mago_atom::Atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta =
    ProviderMeta::new("php::array::compact", "compact", "Returns array with keys from variable names");

/// Provider for the `compact()` function.
///
/// Builds a keyed array with keys being the variable names and values being the variable values.
#[derive(Default)]
pub struct CompactProvider;

impl Provider for CompactProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for CompactProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("compact")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let arguments = invocation.arguments();
        let mut known_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();
        let mut has_unknown = false;

        for invocation_argument in arguments {
            if invocation_argument.is_unpacked() {
                has_unknown = true;
                continue;
            }

            let Some(argument_expr) = invocation_argument.value() else {
                continue;
            };

            let argument_type = context.get_expression_type(argument_expr)?;
            let Some(variable_name) = argument_type.get_single_literal_string_value() else {
                continue;
            };

            let variable_id = format!("${variable_name}");
            if let Some(variable_type) = context.get_variable_type(&variable_id) {
                let key = ArrayKey::String(Atom::from(variable_name));
                known_items.insert(key, (false, (**variable_type).clone()));
            } else {
                has_unknown = true;
            }
        }

        if known_items.is_empty() {
            return None;
        }

        let mut keyed_array = TKeyedArray::new();
        keyed_array.known_items = Some(known_items);
        keyed_array.non_empty = true;
        if has_unknown {
            keyed_array.parameters = Some((Arc::new(get_string()), Arc::new(get_mixed())));
        }

        Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed_array))))
    }
}
