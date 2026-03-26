//! `array_column()` return type provider.

use std::borrow::Cow;
use std::sync::Arc;

use mago_atom::concat_atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
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

        let array_parameters = get_array_parameters(array, codebase);
        let obj = array_parameters.1.get_single_named_object()?;

        let class_like = codebase.get_class_like(&obj.name)?;

        let column_key_argument = invocation.get_argument(1, &["column_key"])?;
        let column_key_type = context.get_expression_type(column_key_argument)?;

        let column_type = if column_key_type.is_null() {
            TUnion::from_atomic(TAtomic::Object(TObject::Named(obj.clone())))
        } else {
            let column_key_property_name = column_key_type.get_single_literal_string_value()?;
            let column_key_property = class_like.properties.get(&concat_atom!("$", column_key_property_name))?;

            column_key_property.type_metadata.as_ref()?.type_union.clone()
        };

        let index_key_argument = invocation.get_argument(2, &["index_key"]);
        let index_key_type = index_key_argument.and_then(|argument| context.get_expression_type(argument));

        let mut index_type = None;
        if let Some(index_key_type) = index_key_type {
            let index_key_property_name = index_key_type.get_single_literal_string_value();
            let index_key_property = index_key_property_name
                .and_then(|property_name| class_like.properties.get(&concat_atom!("$", property_name)));

            if let Some(index_key_property) = index_key_property {
                index_type = match index_key_property.type_metadata.as_ref()?.type_union.get_single() {
                    TAtomic::Scalar(
                        scalar @ (TScalar::ArrayKey
                        | TScalar::Integer(_)
                        | TScalar::String(_)
                        | TScalar::ClassLikeString(_)),
                    ) => Some(scalar),
                    _ => None,
                };
            }
        }

        if let Some(index_type) = index_type {
            let keyed_array = TKeyedArray::new_with_parameters(
                Arc::new(TUnion::from_atomic(TAtomic::Scalar(index_type.clone()))),
                Arc::new(column_type),
            );

            return Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed_array))));
        }

        let list = TList::new(Arc::new(column_type));

        Some(TUnion::from_single(Cow::Owned(TAtomic::Array(TArray::List(list)))))
    }
}
