use std::collections::BTreeMap;
use std::sync::Arc;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;
use crate::visibility::is_visible_from_scope;

static META: ProviderMeta = ProviderMeta::new(
    "php::object::get_object_vars",
    "get_object_vars",
    "Returns the object's properties visible from the calling scope as a keyed array",
);

#[derive(Default)]
pub struct GetObjectVarsProvider;

impl Provider for GetObjectVarsProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for GetObjectVarsProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact(b"get_object_vars")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let argument = invocation.get_argument(0, &[b"object"])?;
        let argument_type = context.get_expression_type(argument)?;
        let object = argument_type.get_single_named_object()?;

        let class_metadata = context.get_class_like(object.name)?;
        if class_metadata.kind.is_enum() {
            return None;
        }

        let current_class = context.current_class_name();

        let mut known_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();
        for (property_name, declaring_class) in class_metadata.appearing_property_ids.iter() {
            let Some(property) = context.codebase().get_property(declaring_class.as_bytes(), property_name.as_bytes())
            else {
                continue;
            };

            if property.flags.is_static() || property.flags.is_virtual_property() {
                continue;
            }

            if !is_visible_from_scope(
                context.codebase(),
                property.read_visibility,
                declaring_class.as_bytes(),
                current_class,
            ) {
                continue;
            }

            let Some(name) = property_name.as_bytes().strip_prefix(b"$") else {
                continue;
            };

            let property_type =
                property.type_metadata.as_ref().map(|metadata| metadata.type_union.clone()).unwrap_or_else(get_mixed);

            known_items.insert(ArrayKey::String(mago_word::word(name)), (false, property_type));
        }

        if known_items.is_empty() {
            return None;
        }

        let mut keyed_array = TKeyedArray::new();
        keyed_array.known_items = Some(known_items);
        keyed_array.non_empty = true;

        if !class_metadata.flags.is_final() || !class_metadata.flags.is_readonly() {
            keyed_array.parameters = Some((Arc::new(get_arraykey()), Arc::new(get_mixed())));
        }

        Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed_array))))
    }
}
