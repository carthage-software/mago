use std::borrow::Cow;
use std::collections::BTreeMap;

use mago_atom::Atom;
use mago_atom::concat_atom;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::get_array_parameters;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::special_function_like_handler::SpecialFunctionLikeHandlerTrait;
use crate::invocation::special_function_like_handler::utils::get_argument;
use crate::visibility::check_property_read_visibility;

#[derive(Debug)]
pub struct ArrayFunctionsHandler;

impl SpecialFunctionLikeHandlerTrait for ArrayFunctionsHandler {
    fn get_return_type<'ctx, 'ast, 'arena>(
        &self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_like_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<TUnion> {
        match function_like_name {
            "array_column" => {
                let array_argument = get_argument(invocation.arguments_source, 0, vec!["array"])?;
                let array_type = artifacts.get_expression_type(array_argument)?;

                let array = array_type.get_single_array()?;

                let array_parameters = get_array_parameters(array, context.codebase);
                let obj = array_parameters.1.get_single_named_object()?;

                let class_like = context.codebase.get_class_like(&obj.name)?;

                let column_key_argument = get_argument(invocation.arguments_source, 1, vec!["column_key"])?;
                let column_key_type = artifacts.get_expression_type(column_key_argument)?;

                let column_type = if !column_key_type.is_null() {
                    let column_key_property_name = column_key_type.get_single_literal_string_value()?;
                    let column_key_property =
                        class_like.properties.get(&concat_atom!("$", column_key_property_name))?;

                    if !check_property_read_visibility(
                        context,
                        block_context,
                        &class_like.name,
                        concat_atom!("$", column_key_property_name).into(),
                        column_key_argument.span(),
                        column_key_property.span,
                    ) {
                        return None;
                    }

                    column_key_property.type_metadata.as_ref()?.type_union.clone()
                } else {
                    TUnion::from_atomic(TAtomic::Object(TObject::Named(obj.clone())))
                };

                let index_key_argument = get_argument(invocation.arguments_source, 2, vec!["index_key"]);
                let index_key_type = index_key_argument.and_then(|argument| artifacts.get_expression_type(argument));

                let mut index_type = None;
                if let (Some(index_key_argument), Some(index_key_type)) = (index_key_argument, index_key_type) {
                    let index_key_property_name = index_key_type.get_single_literal_string_value();
                    let index_key_property = index_key_property_name
                        .and_then(|property_name| class_like.properties.get(&concat_atom!("$", property_name)));

                    if let Some(index_key_property) = index_key_property {
                        if !check_property_read_visibility(
                            context,
                            block_context,
                            &class_like.name,
                            concat_atom!("$", index_key_property.name.0).into(),
                            index_key_argument.span(),
                            index_key_property.span,
                        ) {
                            return None;
                        }

                        index_type = match index_key_property.type_metadata.as_ref()?.type_union.get_single() {
                            TAtomic::Scalar(scalar @ TScalar::ArrayKey)
                            | TAtomic::Scalar(scalar @ TScalar::Integer(_))
                            | TAtomic::Scalar(scalar @ TScalar::String(_))
                            | TAtomic::Scalar(scalar @ TScalar::ClassLikeString(_)) => Some(scalar),
                            _ => None,
                        };
                    }
                }

                if let Some(index_type) = index_type {
                    let keyed_array = TKeyedArray::new_with_parameters(
                        Box::new(TUnion::from_atomic(TAtomic::Scalar(index_type.clone()))),
                        Box::new(column_type),
                    );

                    return Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed_array))));
                }

                let list = TList::new(Box::new(column_type));

                Some(TUnion::from_single(Cow::Owned(TAtomic::Array(TArray::List(list)))))
            }
            "compact" => {
                let arguments = invocation.arguments_source.get_arguments();
                let mut known_items: BTreeMap<ArrayKey, (bool, TUnion)> = BTreeMap::new();

                let mut has_unknown = false;
                for invocation_argument in arguments {
                    // Skip unpacked arguments
                    if invocation_argument.is_unpacked() {
                        has_unknown = true;
                        continue;
                    }

                    let argument_expr = invocation_argument.value();
                    let argument_type = artifacts.get_expression_type(argument_expr)?;

                    // Get the literal string value (variable name)
                    let variable_name = match argument_type.get_single_literal_string_value() {
                        Some(name) => name,
                        None => continue, // Skip non-literal arguments
                    };

                    // Look up the variable in block context
                    // Create variable ID by prepending "$" to the variable name
                    let variable_id = format!("${}", variable_name);
                    if let Some(variable_type) = block_context.locals.get(&variable_id) {
                        // Add to known_items with the variable name as key (convert to Atom)
                        let key = ArrayKey::String(Atom::from(variable_name));
                        known_items.insert(key, (false, (**variable_type).clone()));
                    } else {
                        has_unknown = true;
                    }
                }

                // If we didn't find any items, return None to fall back to default handling
                if known_items.is_empty() {
                    return None;
                }

                // Build the keyed array
                let mut keyed_array = TKeyedArray::new();
                keyed_array.known_items = Some(known_items);
                keyed_array.non_empty = true;
                if has_unknown {
                    keyed_array.parameters = Some((Box::new(get_string()), Box::new(get_mixed())));
                }

                Some(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed_array))))
            }
            _ => None,
        }
    }
}
