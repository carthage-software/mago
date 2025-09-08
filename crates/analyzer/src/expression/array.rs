use std::borrow::Cow;
use std::collections::BTreeMap;

use ahash::HashSet;

use mago_atom::AtomSet;
use mago_atom::atom;
use mago_atom::empty_atom;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::combiner::combine;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::comparator::union_comparator::is_contained_by;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_empty_keyed_array;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_iterable_parameters;
use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::get_literal_string;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

/// Analyzes array literals and their elements.
///
/// Example:
///
/// ```php
/// $array = [
///    'key1' => 'value1',
///    'key2' => 'value2',
/// ];
/// ```
impl<'ast, 'arena> Analyzable<'ast, 'arena> for Array<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_array_elements(context, block_context, artifacts, self.span(), self.elements.as_slice())
    }
}

/// Analyzes a legacy array literal.
///
/// Example:
///
/// ```php
/// $array = array('key1' => 'value1', 'key2' => 'value2');
/// ```
impl<'ast, 'arena> Analyzable<'ast, 'arena> for LegacyArray<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_array_elements(context, block_context, artifacts, self.span(), self.elements.as_slice())
    }
}

#[derive(Debug)]
struct ArrayCreationInfo {
    item_key_atomic_types: Vec<TAtomic>,
    item_value_atomic_types: Vec<TAtomic>,
    property_types: BTreeMap<ArrayKey, (bool, TUnion)>,
    class_strings: AtomSet,
    can_create_objectlike: bool,
    array_keys: HashSet<ArrayKey>,
    int_offset: i64,
    is_list: bool,
    can_be_empty: bool,
}

fn analyze_array_elements<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    expression_span: Span,
    elements: &[ArrayElement<'arena>],
) -> Result<(), AnalysisError> {
    if elements.is_empty() {
        artifacts.set_expression_type(&expression_span, get_empty_keyed_array());

        return Ok(());
    }

    let mut array_creation_info = ArrayCreationInfo {
        item_key_atomic_types: Vec::new(),
        item_value_atomic_types: Vec::new(),
        property_types: BTreeMap::default(),
        class_strings: AtomSet::default(),
        can_create_objectlike: true,
        array_keys: HashSet::default(),
        int_offset: -1,
        is_list: true,
        can_be_empty: true,
    };

    for element in elements {
        let (item_key_value, key_type, item_is_list_item, value) = match element {
            ArrayElement::KeyValue(key_value_array_element) => {
                let was_inside_general_use = block_context.inside_general_use;
                block_context.inside_general_use = true;
                key_value_array_element.key.analyze(context, block_context, artifacts)?;
                block_context.inside_general_use = was_inside_general_use;

                let (item_key_value, key_type) = artifacts
                    .get_expression_type(key_value_array_element.key)
                    .map(|item_key_type| {
                        let key_type = if item_key_type.is_null() {
                            get_literal_string(empty_atom())
                        } else if item_key_type.is_true() {
                            get_literal_int(1)
                        } else if item_key_type.is_false() {
                            get_literal_int(0)
                        } else if let Some(f) = item_key_type.get_single_literal_float_value() {
                            get_literal_int(f.trunc() as i64)
                        } else if item_key_type.is_float() {
                            get_int()
                        } else if !item_key_type.is_always_array_key(true) {
                            let item_key_type_id = item_key_type.get_id();

                            context.collector.report_with_code(
                                IssueCode::InvalidArrayElementKey,
                                Issue::error("Invalid array key type.")
                                    .with_annotation(
                                        Annotation::primary(key_value_array_element.key.span()).with_message(format!(
                                            "This has type `{item_key_type_id}`, which cannot be cast to a string or integer.",
                                        )),
                                    )
                                    .with_note(format!(
                                        "In PHP, array keys must be strings or integers. While types like `bool` or `float` are automatically cast, a value of type `{item_key_type_id}` cannot be.",
                                    ))
                                    .with_help("Ensure the array key is either a string or an integer."),
                            );

                            get_arraykey()
                        } else {
                            item_key_type.clone()
                        };

                        let item_key_value =
                            if let Some(item_key_literal_type) = key_type.get_single_literal_string_value() {
                                let string_to_int = get_numeric_key_from_string(item_key_literal_type);

                                Some(match string_to_int {
                                    Some(integer) => ArrayKey::Integer(integer),
                                    None => ArrayKey::String(atom(item_key_literal_type)),
                                })
                            } else if let Some(literal_integer) = key_type.get_single_literal_int_value() {
                                // The most recent integer key becomes the next available integer key
                                array_creation_info.int_offset = literal_integer;

                                Some(ArrayKey::Integer(literal_integer))
                            } else if let Some(class_string) = key_type.get_single_class_string_value() {
                                array_creation_info.class_strings.insert(class_string);

                                Some(ArrayKey::String(class_string))
                            } else {
                                None
                            };

                        (item_key_value, key_type)
                    })
                    .unwrap_or((None, get_mixed()));

                (item_key_value, key_type, false, key_value_array_element.value)
            }
            ArrayElement::Value(value_array_element) => {
                // check if we have reached PHP_INT_MAX
                if array_creation_info.int_offset == i64::MAX {
                    context.collector.report_with_code(
                        IssueCode::InvalidArrayIndex,
                        Issue::error(
                            "Cannot add array item implicitly; the next available integer key would exceed PHP_INT_MAX."
                        )
                        .with_annotation(
                            Annotation::primary(value_array_element.span())
                                .with_message("Adding this item would result in an invalid integer key.")
                        )
                        .with_note(
                            format!("PHP automatically assigns integer keys starting from the highest previous integer key. The next key would exceed `PHP_INT_MAX` ({}).", i64::MAX)
                        )
                        .with_note(
                            "This usually happens in very large arrays or after using an explicit integer key close to the maximum."
                        )
                        .with_help(
                            "Consider using an explicit string key for this item, restructuring the array, or ensuring previous explicit integer keys are smaller."
                        ),
                    );

                    break;
                }

                array_creation_info.int_offset += 1;

                (
                    Some(ArrayKey::Integer(array_creation_info.int_offset)),
                    get_literal_int(array_creation_info.int_offset),
                    true,
                    value_array_element.value,
                )
            }
            ArrayElement::Variadic(variadic_array_element) => {
                let was_inside_general_use = block_context.inside_general_use;
                block_context.inside_general_use = true;
                variadic_array_element.value.analyze(context, block_context, artifacts)?;
                block_context.inside_general_use = was_inside_general_use;

                match artifacts.get_expression_type(&variadic_array_element.value) {
                    Some(variadic_array_element_type) => {
                        handle_variadic_array_element(
                            context,
                            &mut array_creation_info,
                            variadic_array_element,
                            variadic_array_element_type,
                        );
                    }
                    None => {
                        array_creation_info.can_create_objectlike = false;
                        array_creation_info.item_key_atomic_types.push(TAtomic::Scalar(TScalar::ArrayKey));
                        array_creation_info.item_value_atomic_types.push(TAtomic::Mixed(TMixed::new()));
                    }
                };

                continue;
            }
            ArrayElement::Missing(missing_array_element) => {
                context.collector.report_with_code(
                    IssueCode::InvalidArrayElement,
                    Issue::error(
                        "Missing array element: skipping elements is only allowed in list assignments (destructuring)."
                    )
                    .with_annotation(
                        Annotation::primary(missing_array_element.span())
                            .with_message("Element expected here.")
                    )
                    .with_note(
                        "Array literals require a value for each position (e.g., `[1, null, 3]`) unless used on the left side of an assignment (e.g., `[$a, , $c] = ...;`)."
                    )
                    .with_help("Provide a value for this element (e.g., `null`) or remove the extra comma."),
                );

                continue;
            }
        };

        let was_inside_general_use = block_context.inside_general_use;
        block_context.inside_general_use = true;
        value.analyze(context, block_context, artifacts)?;
        block_context.inside_general_use = was_inside_general_use;

        array_creation_info.can_be_empty = false;
        array_creation_info.is_list &= item_is_list_item;

        if let Some(item_key_value) = item_key_value {
            if array_creation_info.array_keys.contains(&item_key_value) {
                context.collector.report_with_code(
                    IssueCode::DuplicateArrayKey,
                    Issue::error(format!(
                        "Duplicate array key `{item_key_value}` detected."
                    ))
                    .with_annotation(
                        Annotation::primary(element.span())
                            .with_message(format!("This key `{item_key_value}` duplicates an earlier key"))
                    )
                    .with_note(
                        "Using the same key multiple times in an array literal will overwrite the previous value associated with that key."
                    )
                    .with_help(
                        "Remove the duplicate entry or use a unique key for this element."
                    ),
                );
            } else {
                array_creation_info.array_keys.insert(item_key_value);
            }
        }

        if value.is_unary() {
            // TODO(azjezz): handle by ref values: https://github.com/vimeo/psalm/blob/d74446a78f0e8431fb85ef37889927b862aee09c/src/Psalm/Internal/Analyzer/Statements/Expression/ArrayAnalyzer.php#L509-L528
        }

        match artifacts.get_expression_type(value) {
            Some(value_type) => {
                if let Some(item_key_value) = item_key_value {
                    array_creation_info.property_types.insert(item_key_value, (false, value_type.clone()));
                } else {
                    array_creation_info.can_create_objectlike = false;
                    array_creation_info.item_key_atomic_types.extend(key_type.types.into_owned());
                    array_creation_info.item_value_atomic_types.extend(value_type.types.iter().cloned());
                }
            }
            None => {
                if let Some(item_key_value) = item_key_value {
                    array_creation_info.property_types.insert(item_key_value, (false, get_mixed()));
                } else {
                    array_creation_info.can_create_objectlike = false;
                    array_creation_info.item_key_atomic_types.extend(key_type.types.into_owned());
                    array_creation_info.item_value_atomic_types.push(TAtomic::Mixed(TMixed::new()));
                }
            }
        }
    }

    let item_key_type = if !array_creation_info.item_key_atomic_types.is_empty() {
        Some(TUnion::from_vec(combine(array_creation_info.item_key_atomic_types, context.codebase, false)))
    } else {
        None
    };

    let item_value_type = if !array_creation_info.item_value_atomic_types.is_empty() {
        Some(TUnion::from_vec(combine(array_creation_info.item_value_atomic_types, context.codebase, false)))
    } else {
        None
    };

    let array_type = if !array_creation_info.property_types.is_empty() {
        if array_creation_info.is_list {
            TUnion::from_vec(vec![TAtomic::Array(TArray::List(TList {
                known_count: Some(array_creation_info.property_types.len()),
                known_elements: Some(BTreeMap::from_iter(
                    array_creation_info
                        .property_types
                        .into_iter()
                        .enumerate()
                        .map(|(index, (_, value_tuple))| (index, (value_tuple.0, value_tuple.1))),
                )),
                element_type: Box::new(match item_value_type {
                    Some(value) => value,
                    None => get_never(),
                }),
                non_empty: true,
            }))])
        } else {
            TUnion::from_vec(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(BTreeMap::from_iter(
                    array_creation_info.property_types.into_iter().map(|(k, v)| (k, (v.0, v.1))),
                )),
                parameters: if array_creation_info.can_create_objectlike {
                    None
                } else {
                    match (item_key_type, item_value_type) {
                        (Some(key), Some(value)) => Some((Box::new(key), Box::new(value))),
                        (Some(key), None) => Some((Box::new(key), Box::new(get_mixed()))),
                        (None, Some(value)) => Some((Box::new(get_arraykey()), Box::new(value))),
                        _ => Some((Box::new(get_arraykey()), Box::new(get_mixed()))),
                    }
                },
                non_empty: true,
            }))])
        }
    } else if item_key_type.is_none() && item_value_type.is_none() {
        get_empty_keyed_array()
    } else if array_creation_info.is_list {
        TUnion::from_vec(vec![TAtomic::Array(TArray::List(TList {
            known_elements: None,
            element_type: Box::new(item_value_type.unwrap_or_else(get_mixed)),
            known_count: None,
            non_empty: !array_creation_info.can_be_empty,
        }))])
    } else {
        TUnion::from_vec(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
            known_items: None,
            parameters: match (item_key_type, item_value_type) {
                (Some(key), Some(value)) => Some((Box::new(key), Box::new(value))),
                (Some(key), None) => Some((Box::new(key), Box::new(get_mixed()))),
                (None, Some(value)) => Some((Box::new(get_arraykey()), Box::new(value))),
                _ => Some((Box::new(get_arraykey()), Box::new(get_mixed()))),
            },
            non_empty: !array_creation_info.can_be_empty,
        }))])
    };

    artifacts.set_expression_type(&expression_span, array_type);

    Ok(())
}

fn get_numeric_key_from_string(key: &str) -> Option<i64> {
    if key.starts_with("0") || key.starts_with("+") {
        return None;
    }

    if key.trim() != key {
        return None;
    }

    key.parse::<i64>().ok()
}

fn handle_variadic_array_element<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    array_creation_info: &mut ArrayCreationInfo,
    variadic_array_element: &'ast VariadicArrayElement<'arena>,
    variadic_array_element_type: &TUnion,
) {
    let mut all_non_empty = true;

    for atomic_type in variadic_array_element_type.types.as_ref() {
        let (key_type, value_type) = match atomic_type {
            TAtomic::Array(array_type) => match array_type {
                TArray::Keyed(keyed_data) => {
                    if let Some(known_items) = &keyed_data.known_items {
                        for (key, (possibly_undefined, value_type)) in known_items {
                            if *possibly_undefined {
                                continue;
                            }

                            let new_offset_key = match key {
                                ArrayKey::Integer(_) => {
                                    if array_creation_info.int_offset == i64::MAX {
                                        context.collector.report_with_code(
                                            IssueCode::InvalidArrayIndex,
                                            Issue::error(
                                                "Cannot add an item with an offset beyond `PHP_INT_MAX`."
                                            )
                                            .with_annotation(
                                                Annotation::primary(variadic_array_element.span())
                                                    .with_message("Adding this item would result in an invalid integer key.")
                                            )
                                            .with_note(
                                                format!("PHP automatically assigns integer keys starting from the highest previous integer key. The next key would exceed `PHP_INT_MAX` ({}).", i64::MAX)
                                            )
                                            .with_note(
                                                "This usually happens in very large arrays or after using an explicit integer key close to the maximum."
                                            )
                                            .with_help(
                                                "Consider using an explicit string key for this item, restructuring the array, or ensuring previous explicit integer keys are smaller."
                                            ),
                                        );

                                        continue;
                                    }

                                    array_creation_info.int_offset += 1;
                                    array_creation_info
                                        .item_key_atomic_types
                                        .push(TAtomic::Scalar(TScalar::literal_int(array_creation_info.int_offset)));

                                    ArrayKey::Integer(array_creation_info.int_offset)
                                }
                                ArrayKey::String(string_key) => {
                                    array_creation_info.is_list = false;
                                    array_creation_info
                                        .item_key_atomic_types
                                        .push(TAtomic::Scalar(TScalar::String(TString::known_literal(*string_key))));
                                    ArrayKey::String(*string_key)
                                }
                            };

                            array_creation_info.array_keys.insert(new_offset_key);
                            array_creation_info.property_types.insert(new_offset_key, (false, value_type.clone()));
                        }
                    }

                    // Update non-empty status
                    if !keyed_data.non_empty {
                        all_non_empty = false;
                    }

                    match keyed_data.get_generic_parameters() {
                        Some(parameters) => (Some(Cow::Borrowed(parameters.0)), Cow::Borrowed(parameters.1)),
                        None => {
                            continue;
                        }
                    }
                }
                TArray::List(list_data) => {
                    // Process known elements
                    if let Some(known_elements) = &list_data.known_elements {
                        // Original logic iterated values(), not keys. Let's adjust to match the old intent if needed.
                        // Assuming the goal IS to add elements sequentially based on the values in the spread list:
                        for (definite, value_type) in known_elements.values() {
                            // Key _idx ignored if appending
                            if !*definite {
                                continue;
                            }

                            if array_creation_info.int_offset == i64::MAX {
                                context.collector.report_with_code(
                                    IssueCode::InvalidArrayIndex,
                                    Issue::error(
                                        "Cannot add an item with an offset beyond `PHP_INT_MAX`."
                                    )
                                    .with_annotation(
                                        Annotation::primary(variadic_array_element.span())
                                            .with_message("Adding this item would result in an invalid integer key.")
                                    )
                                    .with_note(
                                        format!("PHP automatically assigns integer keys starting from the highest previous integer key. The next key would exceed `PHP_INT_MAX` ({}).", i64::MAX)
                                    )
                                    .with_note(
                                        "This usually happens in very large arrays or after using an explicit integer key close to the maximum."
                                    )
                                    .with_help(
                                        "Consider using an explicit string key for this item, restructuring the array, or ensuring previous explicit integer keys are smaller."
                                    ),
                                );

                                continue;
                            }

                            array_creation_info.int_offset += 1;
                            let new_key = ArrayKey::Integer(array_creation_info.int_offset);
                            array_creation_info.array_keys.insert(new_key);
                            array_creation_info
                                .item_key_atomic_types
                                .push(TAtomic::Scalar(TScalar::literal_int(array_creation_info.int_offset)));
                            array_creation_info.property_types.insert(new_key, (false, value_type.clone()));
                        }
                    }

                    if !list_data.non_empty {
                        all_non_empty = false;
                    }

                    (None, Cow::Borrowed(list_data.get_element_type()))
                }
            },
            atomic => {
                all_non_empty = false;

                let Some((iterable_key, iterable_value)) = get_iterable_parameters(atomic, context.codebase) else {
                    array_creation_info.can_create_objectlike = false;
                    array_creation_info.item_key_atomic_types.push(TAtomic::Scalar(TScalar::ArrayKey));
                    array_creation_info.item_value_atomic_types.push(TAtomic::Mixed(TMixed::new()));

                    context.collector.report_with_code(
                        IssueCode::InvalidArrayElement,
                        Issue::error(format!(
                            "Cannot use spread operator on non-iterable type `{}`.",
                            atomic.get_id()
                        ))
                        .with_annotation(
                            Annotation::primary(variadic_array_element.span())
                                .with_message("Spread operator requires an iterable type.")
                        )
                        .with_note(
                            "The spread operator (`...`) can only be used with arrays or objects implementing the `Traversable` interface."
                        )
                        .with_help("Consider using an array or a traversable object."),
                    );

                    continue;
                };

                if iterable_value.is_never() {
                    continue;
                }

                array_creation_info.can_create_objectlike = false;

                (Some(Cow::Owned(iterable_key)), Cow::Owned(iterable_value))
            }
        };

        if value_type.is_never() {
            continue;
        }

        if let Some(key_type) = key_type {
            if key_type.is_never() {
                continue;
            }

            let is_string_key = union_comparator::is_contained_by(
                context.codebase,
                &key_type,
                &get_string(),
                false,
                false,
                false,
                &mut ComparisonResult::new(),
            );

            let is_array_key_key = is_string_key
                || union_comparator::is_contained_by(
                    context.codebase,
                    &key_type,
                    &get_arraykey(),
                    false,
                    false,
                    false,
                    &mut ComparisonResult::new(),
                );

            if is_string_key {
                array_creation_info.is_list = false;

                if !context.settings.version.is_at_least(0, 0, 0) {
                    context.collector.report_with_code(
                        IssueCode::InvalidArrayElementKey,
                        Issue::error("String keys are not supported in unpacked arrays")
                            .with_annotation(
                                Annotation::primary(variadic_array_element.span())
                                    .with_message("Spread operator requires an iterable type with array-key keys."),
                            )
                            .with_note(
                                "In PHP versions prior to 8.1, using string keys in unpacked arrays is not supported.",
                            )
                            .with_help("Consider using an array or a traversable object with integer keys."),
                    );

                    continue;
                }
            }

            if !is_array_key_key {
                context.collector.report_with_code(
                    IssueCode::InvalidArrayElementKey,
                    Issue::error(format!(
                        "Cannot use spread operator on an iterable with key type `{}`.",
                        key_type.get_id()
                    ))
                    .with_annotation(
                        Annotation::primary(variadic_array_element.span())
                            .with_message("Spread operator requires an iterable type with array-key keys.")
                    )
                    .with_note(
                        "The spread operator (`...`) can only be used with arrays or objects implementing the `Traversable` interface that have keys of type `array-key`."
                    )
                    .with_help("Consider using an array or a traversable object with appropriate key types.")
                );

                continue;
            }

            for (k, v) in array_creation_info.property_types.iter_mut() {
                let prop_key = k.to_union();

                if is_contained_by(
                    context.codebase,
                    &prop_key,
                    &key_type,
                    false,
                    false,
                    false,
                    &mut ComparisonResult::new(),
                ) {
                    let new_prop_val = combine_union_types(&v.1, &value_type, context.codebase, false);

                    *v = (v.0, new_prop_val);
                }
            }

            array_creation_info.item_key_atomic_types.extend(key_type.into_owned().types.into_owned());
        }

        array_creation_info.item_value_atomic_types.extend(value_type.into_owned().types.into_owned());
    }

    if all_non_empty {
        array_creation_info.can_be_empty = false;
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = array_literal_empty_square_brackets,
        code = indoc! {r#"
            <?php

            /**
             * @param array{} $_arr
             */
            function expect_empty_array(array $_arr): void {}

            $empty1 = [];
            expect_empty_array($empty1);
        "#}
    }

    test_analysis! {
        name = array_literal_empty_array_keyword,
        code = indoc! {r#"
            <?php

            /**
             * @param array{} $_arr
             */
            function expect_empty_array(array $_arr): void {}

            $empty2 = array();
            expect_empty_array($empty2);
        "#}
    }

    test_analysis! {
        name = array_literal_literal_int_keys_as_list,
        code = indoc! {r#"
            <?php

            /**
             * @param list<string> $_arr
             */
            function expect_list_of_strings(array $_arr): void {}

            $arr = [0 => "a", 1 => "b"];
            expect_list_of_strings($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_literal_string_keys,
        code = indoc! {r#"
            <?php

            /**
             * @param array{"name": string, "age": int} $_arr
             */
            function expect_shape_person(array $_arr): void {}

            $arr = ["name" => "Alice", "age" => 30];
            expect_shape_person($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_numeric_string_keys_coerced_to_int,
        code = indoc! {r#"
            <?php

            /**
             * @param array{123: bool, 456: bool} $_arr
             */
            function expect_numeric_string_keys(array $_arr): void {}

            $arr = ["123" => true, "456" => false];
            expect_numeric_string_keys($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_null_key_becomes_empty_string,
        code = indoc! {r#"
            <?php

            /**
             * @param array{"": string} $_arr
             */
            function expect_null_key(array $_arr): void {}

            $arr = [null => "val_for_empty_string_key"];
            expect_null_key($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_bool_keys_coerced_to_int,
        code = indoc! {r#"
            <?php

            /**
             * @param array{1: string, 0: string} $_arr
             */
            function expect_bool_keys(array $_arr): void {}

            $arr = [true => "is_true", false => "is_false"];
            expect_bool_keys($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_float_keys_truncated_to_int,
        code = indoc! {r#"
            <?php

            /**
             * @param array{1: string, 2: string} $_arr
             */
            function expect_float_keys(array $_arr): void {}

            $arr = [1.7 => "val1", 2.0 => "val2"];
            expect_float_keys($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_duplicate_literal_int_key,
        code = indoc! {r#"
            <?php

            /**
             * @param array{0: string} $_arr
             */
            function expect_single_string_at_0(array $_arr):void{}

            $arr = [0 => 'a', 0 => 'b'];
            expect_single_string_at_0($arr);
        "#},
        issues = [IssueCode::DuplicateArrayKey]
    }

    test_analysis! {
        name = array_literal_duplicate_literal_string_key,
        code = indoc! {r#"
            <?php

            /**
             * @param array{"k":string} $_arr
             */
            function expect_k_string(array $_arr):void{}

            $arr = ["k" => 'a', "k" => 'b'];
            expect_k_string($arr);
        "#},
        issues = [IssueCode::DuplicateArrayKey]
    }

    test_analysis! {
        name = array_literal_duplicate_coerced_key,
        code = indoc! {r#"
            <?php

            /**
             * @param array{1:string} $_arr
             */
            function expect_one_string(array $_arr):void{}

            $arr = [true => 'a', 1 => 'b']; // true becomes 1, so '1' is duplicated
            expect_one_string($arr);
        "#},
        issues = [IssueCode::DuplicateArrayKey]
    }

    test_analysis! {
        name = array_literal_simple_list_implicit_keys,
        code = indoc! {r#"
            <?php

            /** @param list<string> $_arr */
            function expect_list_of_strings(array $_arr): void {}

            $arr = ["alpha", "beta", "gamma"];
            expect_list_of_strings($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_mixed_implicit_explicit_keys_maintaining_list,
        code = indoc! {r#"
            <?php

            /** @param array{0: string, 1: string, 2: string} $_arr */
            function expect_shape_list_three_strings(array $_arr): void {}

            $arr = [0 => "a", "b", 2 => "c"];
            expect_shape_list_three_strings($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_mixed_implicit_explicit_keys_breaking_list,
        code = indoc! {r#"
            <?php

            /** @param array<array-key, string> $_arr */
            function expect_general_keyed_array_string_values(array $_arr): void {}

            $arr = ["key" => "a", "b"]; // "b" gets key 0, becomes keyed array
            expect_general_keyed_array_string_values($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_spread_empty_array,
        code = indoc! {r#"
            <?php

            /** @param array{} $_arr */
            function expect_empty_array(array $_arr): void {}

            $empty = [];
            $arr = [...$empty];
            expect_empty_array($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_spread_list,
        code = indoc! {r#"
            <?php

            /** @param list<string> $_arr */
            function expect_list_of_strings(array $_arr): void {}

            $list = ["a", "b"];
            $arr = [...$list, "c"];
            expect_list_of_strings($arr); // Expects list<string>
        "#}
    }

    test_analysis! {
        name = array_literal_spread_keyed_array,
        code = indoc! {r#"
            <?php

            /** @param array<string, int> $_arr */
            function expect_map_string_to_int(array $_arr): void {}

            $keyed = ["x" => 1, "y" => 2];
            $arr = [...$keyed, "z" => 3];
            expect_map_string_to_int($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_spread_non_iterable_int,
        code = indoc! {r#"
            <?php

            /** @param array<array-key, mixed> $_arr */
            function expect_empty_array(array $_arr): void {}

            $num = 123;
            $arr = [...$num];
            expect_empty_array($arr);
        "#},
        issues = [IssueCode::InvalidArrayElement]
    }

    test_analysis! {
        name = array_literal_missing_element,
        code = indoc! {r#"
            <?php

            $arr = [1, , 3];
        "#},
        issues = [IssueCode::InvalidArrayElement]
    }

    test_analysis! {
        name = array_literal_results_in_list_type,
        code = indoc! {r#"
            <?php

            /** @param list<int> $_arr */
            function expect_list_of_ints(array $_arr): void {}

            $arr = [10, 20, 30];
            expect_list_of_ints($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_results_in_keyed_array_string_keys,
        code = indoc! {r#"
            <?php

            /** @param array<string, bool> $_arr */
            function expect_map_string_to_bool(array $_arr): void {}

            $arr = ["a" => true, "b" => false];
            expect_map_string_to_bool($arr);
        "#}
    }

    test_analysis! {
        name = array_literal_results_in_keyed_array_mixed_keys,
        code = indoc! {r#"
            <?php

            /** @param array<array-key, int> $_arr */
            function expect_general_keyed_array_int_values(array $_arr): void {}

            $arr = [0 => 1, "a" => 2, 1 => 3];
            expect_general_keyed_array_int_values($arr);
        "#}
    }

    test_analysis! {
        name = list_array_comparison,
        code = indoc! {r#"
            <?php

            /**
             * @return array{0: 1, 1: 2, 2: 3, 3: 4, 4: 5}
             */
            function x(): array {
               exit();
            }

            /**
             * @param list<1|2|3|4|5> $_list
             */
            function take_list_ints(array $_list): void {
            }

            $list = x();
            take_list_ints($list);
        "#}
    }

    test_analysis! {
        name = create_array_using_generic_key,
        code = indoc! {r#"
            <?php

            /**
             * @template K of array-key
             * @template V
             *
             * @param K $k
             * @param V $v
             *
             * @return non-empty-array<K, V>
             */
            function create_array($k, $v): array {
                return [$k => $v];
            }
        "#},
    }
}
