//! `array_column()` return type provider.

use std::borrow::Cow;
use std::sync::Arc;

use mago_atom::atom;
use mago_atom::concat_atom;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::get_array_parameters;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

static META: ProviderMeta = ProviderMeta::new(
    "php::array::array_column",
    "array_column",
    "Returns list or array based on column_key and index_key arguments",
);

/// Provider for the `array_column()` function.
///
/// Returns typed arrays based on the `column_key` and `index_key` arguments.
#[derive(Default)]
pub struct ArrayColumnProvider;

impl Provider for ArrayColumnProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl FunctionReturnTypeProvider for ArrayColumnProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::Exact("array_column")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let array_argument = invocation.get_argument(0, &["array"])?;
        let array_type = context.get_expression_type(array_argument)?;

        let array = array_type.get_single_array()?;
        let codebase = context.codebase();
        let element_type = get_array_parameters(array, codebase).1;

        let column_key_argument = invocation.get_argument(1, &["column_key"])?;
        let column_key_type = context.get_expression_type(column_key_argument)?;

        let index_key_argument = invocation.get_argument(2, &["index_key"]);
        let index_key_type = index_key_argument.and_then(|arg| context.get_expression_type(arg));

        if let Some(result) = try_resolve_from_named_object(&element_type, column_key_type, index_key_type, codebase) {
            return Some(result);
        }

        if let Some(result) = try_resolve_from_keyed_array(&element_type, column_key_type, index_key_type) {
            return Some(result);
        }

        None
    }
}

/// Resolve column and index types from an object element type by looking up
/// class properties.
fn try_resolve_from_named_object(
    element_type: &TUnion,
    column_key_type: &TUnion,
    index_key_type: Option<&TUnion>,
    codebase: &mago_codex::metadata::CodebaseMetadata,
) -> Option<TUnion> {
    let obj = element_type.get_single_named_object()?;
    let class_like = codebase.get_class_like(&obj.name)?;

    let column_type = if column_key_type.is_null() {
        TUnion::from_atomic(TAtomic::Object(TObject::Named(obj.clone())))
    } else {
        let prop_name = column_key_type.get_single_literal_string_value()?;
        let prop = class_like.properties.get(&concat_atom!("$", prop_name))?;
        prop.type_metadata.as_ref()?.type_union.clone()
    };

    let index_type = resolve_index_type_from_property(index_key_type, class_like);

    Some(build_result(column_type, index_type))
}

/// Resolve column and index types from a keyed-array element type by looking
/// up known items.
fn try_resolve_from_keyed_array(
    element_type: &TUnion,
    column_key_type: &TUnion,
    index_key_type: Option<&TUnion>,
) -> Option<TUnion> {
    if !element_type.is_single() {
        return None;
    }

    let TAtomic::Array(TArray::Keyed(keyed)) = element_type.get_single() else {
        return None;
    };

    let known_items = keyed.get_known_items()?;

    let column_type = if column_key_type.is_null() {
        element_type.clone()
    } else {
        let key_str = column_key_type.get_single_literal_string_value()?;
        let (_, value_type) = known_items.get(&ArrayKey::String(atom(key_str)))?;
        value_type.clone()
    };

    let index_type = if let Some(index_key_type) = index_key_type {
        if index_key_type.is_null() {
            None
        } else {
            let key_str = index_key_type.get_single_literal_string_value()?;
            let (_, value_type) = known_items.get(&ArrayKey::String(atom(key_str)))?;
            extract_scalar_for_key(value_type)
        }
    } else {
        None
    };

    Some(build_result(column_type, index_type))
}

/// Try to extract the key scalar type from a value type (for use as array index).
fn extract_scalar_for_key(value_type: &TUnion) -> Option<&TScalar> {
    if !value_type.is_single() {
        return None;
    }

    match value_type.get_single() {
        TAtomic::Scalar(
            scalar @ (TScalar::ArrayKey | TScalar::Integer(_) | TScalar::String(_) | TScalar::ClassLikeString(_)),
        ) => Some(scalar),
        _ => None,
    }
}

fn resolve_index_type_from_property<'a>(
    index_key_type: Option<&TUnion>,
    class_like: &'a ClassLikeMetadata,
) -> Option<&'a TScalar> {
    let index_key_type = index_key_type?;
    if index_key_type.is_null() {
        return None;
    }

    let prop_name = index_key_type.get_single_literal_string_value()?;
    let prop = class_like.properties.get(&concat_atom!("$", prop_name))?;
    let prop_type = &prop.type_metadata.as_ref()?.type_union;

    extract_scalar_for_key(prop_type)
}

fn build_result(column_type: TUnion, index_type: Option<&TScalar>) -> TUnion {
    if let Some(index_scalar) = index_type {
        let keyed_array = TKeyedArray::new_with_parameters(
            Arc::new(TUnion::from_atomic(TAtomic::Scalar(index_scalar.clone()))),
            Arc::new(column_type),
        );

        TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed_array)))
    } else {
        let list = TList::new(Arc::new(column_type));

        TUnion::from_single(Cow::Owned(TAtomic::Array(TArray::List(list))))
    }
}
