//! `array_column()` return type provider.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_array_parameters;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::cst::Expression;
use mago_word::concat_word;
use mago_word::word;

use crate::code::IssueCode;
use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;
use crate::resolver::property::get_enum_builtin_property_type;
use crate::resolver::property::resolve_property_for_external_access;

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
        FunctionTarget::Exact(b"array_column")
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        let array_argument = invocation.get_argument(0, &[b"array"])?;
        let array_type = context.get_expression_type(array_argument)?;

        let array = array_type.get_single_array()?;
        let codebase = context.codebase();
        let element_type = get_array_parameters(array, codebase).1;

        let column_key_argument = invocation.get_argument(1, &[b"column_key"])?;
        let column_key_type = context.get_expression_type(column_key_argument)?;

        let index_key_argument = invocation.get_argument(2, &[b"index_key"]);
        let index_key_type = index_key_argument.and_then(|arg| context.get_expression_type(arg));

        if let Some(result) = try_resolve_from_objects(&element_type, column_key_type, index_key_type, codebase) {
            return Some(result);
        }

        if let Some(result) =
            try_resolve_from_keyed_array(context, &element_type, column_key_type, index_key_argument, index_key_type)
        {
            return Some(result);
        }

        None
    }
}

/// Resolve column and index types from object element types by looking up their properties.
fn try_resolve_from_objects(
    element_type: &TUnion,
    column_key_type: &TUnion,
    index_key_type: Option<&TUnion>,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    let index_key_type = index_key_type.filter(|index_key_type| !index_key_type.is_null());

    let mut column_type = None;
    let mut index_type = None;
    for atomic in element_type.types.iter() {
        let TAtomic::Object(object) = atomic else {
            return None;
        };

        let object_column_type = if column_key_type.is_null() {
            TUnion::from_atomic(atomic.clone())
        } else {
            object_property_type(object, column_key_type.get_single_literal_string_value()?, codebase)?
        };
        column_type = Some(add_optional_union_type(object_column_type, column_type.as_ref(), codebase));

        if let Some(index_key_type) = index_key_type {
            let object_index_type = index_key_type
                .get_single_literal_string_value()
                .and_then(|property_name| object_property_type(object, property_name, codebase))
                .map_or_else(get_arraykey, |property_type| key_type_for_index(&property_type));
            index_type = Some(add_optional_union_type(object_index_type, index_type.as_ref(), codebase));
        }
    }

    Some(build_result(column_type?, index_type))
}

fn object_property_type(object: &TObject, property_name: &[u8], codebase: &CodebaseMetadata) -> Option<TUnion> {
    match object {
        TObject::Named(named) => {
            let class_like = codebase.get_class_like(named.name.as_bytes())?;

            let property =
                resolve_property_for_external_access(codebase, class_like, concat_word!(b"$", property_name))?;
            let mut property_type = property.declared_type(codebase);
            expander::expand_union(
                codebase,
                &mut property_type,
                &TypeExpansionOptions { self_class: Some(property.declaring_class.name), ..Default::default() },
            );

            Some(property_type)
        }
        TObject::Enum(r#enum) => get_enum_builtin_property_type(codebase, r#enum, property_name),
        _ => None,
    }
}

/// Resolve column and index types from a keyed-array element type by looking
/// up known items.
fn try_resolve_from_keyed_array(
    context: &ProviderContext<'_, '_, '_>,
    element_type: &TUnion,
    column_key_type: &TUnion,
    index_key_argument: Option<&Expression<'_>>,
    index_key_type: Option<&TUnion>,
) -> Option<TUnion> {
    if !element_type.is_single() {
        return None;
    }

    let (known_items, sealed) = match element_type.get_single() {
        TAtomic::Array(TArray::Keyed(keyed)) => (Cow::Borrowed(keyed.get_known_items()?), keyed.parameters.is_none()),
        TAtomic::Array(TArray::List(list)) => (
            Cow::Owned(
                list.known_elements
                    .as_deref()?
                    .iter()
                    .map(|(index, entry)| (ArrayKey::Integer(*index as i64), entry.clone()))
                    .collect::<BTreeMap<_, _>>(),
            ),
            list.element_type.is_never(),
        ),
        _ => return None,
    };

    if let Some(index_key_argument) = index_key_argument
        && let Some((index_optional, index_value_type)) = index_key_type
            .filter(|index_key_type| !index_key_type.is_null())
            .and_then(literal_array_key)
            .and_then(|key| known_items.get(&key))
    {
        let invalid_count = index_value_type
            .types
            .iter()
            .filter(|atomic| matches!(atomic, TAtomic::Array(_) | TAtomic::Object(_)))
            .count();

        let (rows_may_be_skipped, rows_are_all_skipped) = if column_key_type.is_null() {
            (false, false)
        } else {
            match literal_array_key(column_key_type).map(|key| known_items.get(&key)) {
                Some(Some((optional, _))) => (*optional, false),
                Some(None) => (true, sealed),
                None => (true, false),
            }
        };

        if invalid_count > 0 && !rows_are_all_skipped {
            let all_invalid = !rows_may_be_skipped && !index_optional && invalid_count == index_value_type.types.len();
            let index_type_id = index_value_type.get_id();

            if all_invalid {
                context.report(
                    IssueCode::InvalidArgument,
                    Issue::error(format!(
                        "The index column of `array_column()` has the type `{index_type_id}`, which cannot be an array key."
                    ))
                    .with_annotation(
                        Annotation::primary(index_key_argument.span())
                            .with_message(format!("This column holds `{index_type_id}`")),
                    )
                    .with_note("Since PHP 8, `array_column()` throws a `TypeError` for an array or object in the index column, even a `Stringable` one.")
                    .with_help("Pick an index column that holds strings or ints, or convert the values first, for example with `array_map()`."),
                );

                return Some(get_never());
            }

            context.report(
                IssueCode::PossiblyInvalidArgument,
                Issue::error(format!(
                    "The index column of `array_column()` has the type `{index_type_id}`, which may not be an array key."
                ))
                .with_annotation(
                    Annotation::primary(index_key_argument.span())
                        .with_message(format!("This column may hold an array or object (`{index_type_id}`)")),
                )
                .with_note("Since PHP 8, `array_column()` throws a `TypeError` for an array or object in the index column, even a `Stringable` one.")
                .with_help("Make sure the index column only holds strings or ints, or convert the values first."),
            );
        }
    }

    let column_type = if column_key_type.is_null() {
        element_type.clone()
    } else {
        let (_, value_type) = known_items.get(&literal_array_key(column_key_type)?)?;
        value_type.clone()
    };

    let index_type = index_key_type.filter(|index_key_type| !index_key_type.is_null()).map(|index_key_type| {
        literal_array_key(index_key_type)
            .and_then(|key| known_items.get(&key))
            .map_or_else(get_arraykey, |(_, value_type)| key_type_for_index(value_type))
    });

    Some(build_result(column_type, index_type))
}

fn literal_array_key(key_type: &TUnion) -> Option<ArrayKey> {
    key_type
        .get_single_literal_string_value()
        .map(|key| ArrayKey::String(word(key)))
        .or_else(|| key_type.get_single_literal_int_value().map(ArrayKey::Integer))
}

/// The array key PHP makes from an index column value.
fn key_type_for_index(value_type: &TUnion) -> TUnion {
    let keys: Vec<&TAtomic> =
        value_type.types.iter().filter(|atomic| !matches!(atomic, TAtomic::Array(_) | TAtomic::Object(_))).collect();
    if keys.is_empty() {
        return get_arraykey();
    }

    TUnion::from_vec(
        keys.into_iter()
            .map(|atomic| match atomic {
                TAtomic::Scalar(
                    TScalar::ArrayKey | TScalar::Integer(_) | TScalar::String(_) | TScalar::ClassLikeString(_),
                ) => atomic.clone(),
                TAtomic::Scalar(TScalar::Float(_) | TScalar::Bool(_)) => TAtomic::Scalar(TScalar::int()),
                TAtomic::Null => TAtomic::Scalar(TScalar::literal_string(word(""))),
                _ => TAtomic::Scalar(TScalar::ArrayKey),
            })
            .collect(),
    )
}

fn build_result(column_type: TUnion, index_type: Option<TUnion>) -> TUnion {
    if let Some(index_type) = index_type {
        let keyed_array = TKeyedArray::new_with_parameters(Arc::new(index_type), Arc::new(column_type));

        TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed_array)))
    } else {
        let list = TList::new(Arc::new(column_type));

        TUnion::from_single(Cow::Owned(TAtomic::Array(TArray::List(list))))
    }
}
