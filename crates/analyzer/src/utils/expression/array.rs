use std::borrow::Cow;

use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::data_flow::path::PathKind;
use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::add_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TStringLiteral;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator::is_contained_by;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_mixed_maybe_from_loop;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::get_null;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::get_expression_range;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::issue::TypingIssueKind;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ArrayTarget<'a> {
    Access(&'a ArrayAccess),
    Append(&'a ArrayAppend),
}

impl<'a> ArrayTarget<'a> {
    #[inline]
    pub const fn get_array(&self) -> &'a Expression {
        match self {
            ArrayTarget::Access(array_access) => &array_access.array,
            ArrayTarget::Append(array_append) => &array_append.array,
        }
    }

    #[inline]
    pub const fn get_index(&self) -> Option<&'a Expression> {
        match self {
            ArrayTarget::Access(array_access) => Some(&array_access.index),
            ArrayTarget::Append(_) => None,
        }
    }
}

impl HasSpan for ArrayTarget<'_> {
    fn span(&self) -> Span {
        match self {
            ArrayTarget::Access(array_access) => array_access.span(),
            ArrayTarget::Append(array_append) => array_append.span(),
        }
    }
}

impl<'a> From<&'a ArrayAccess> for ArrayTarget<'a> {
    fn from(array_access: &'a ArrayAccess) -> Self {
        ArrayTarget::Access(array_access)
    }
}

impl<'a> From<&'a ArrayAppend> for ArrayTarget<'a> {
    fn from(array_append: &'a ArrayAppend) -> Self {
        ArrayTarget::Append(array_append)
    }
}

pub(crate) fn get_array_target_type_given_index(
    context: &mut Context<'_>,
    block_context: &mut BlockContext,
    artifacts: &mut AnalysisArtifacts,
    access_span: Span,
    access_array_span: Span,
    access_index_span: Option<Span>,
    array_type: &TUnion,
    mut index_type: &TUnion,
    in_assignment: bool,
    extended_var_id: &Option<String>,
) -> TUnion {
    let mut has_valid_expected_index = false;

    index_type = if let Some(as_type) = index_type.get_generic_parameter_constraint() {
        if as_type.is_mixed() { index_type } else { as_type }
    } else {
        index_type
    };

    let access_index_span = match access_index_span {
        Some(index) => index,
        None => access_span,
    };

    if array_type.is_never() || index_type.is_never() {
        return get_never();
    }

    if index_type.is_null() {
        context.buffer.report(
            TypingIssueKind::NullArrayIndex,
            Issue::error(format!(
                "Cannot use `null` as an array index to access element{}.",
                match extended_var_id {
                    Some(var) => "of variable ".to_string() + var,
                    None => "".to_string(),
                }
            ))
            .with_annotation(
                Annotation::primary(access_index_span).with_message("Index is `null` here.")
            )
            .with_note("Using `null` as an array key is equivalent to using an empty string `''`.")
            .with_help("Ensure the index is an integer or a string. If accessing the key `''` is intended, use an empty string explicitly."),
        );
    }

    if index_type.is_nullable() {
        context.buffer.report(
            TypingIssueKind::PossiblyNullArrayIndex,
            Issue::warning(format!(
                "Possibly using `null` as an array index to access element{}.",
                match extended_var_id {
                    Some(var) => "of variable ".to_string() + var,
                    None => "".to_string(),
                }
            ))
            .with_annotation(Annotation::primary(access_index_span).with_message("Index might be `null` here."))
            .with_note("Using `null` as an array key is equivalent to using an empty string `''`.")
            .with_note("The analysis indicates this index could be `null` at runtime.")
            .with_help("Ensure the index is always an integer or a string, potentially using checks or assertions before access."),
        );
    }

    let mut array_atomic_types = array_type.types.iter().collect::<Vec<_>>();

    let mut value_type = None;
    while let Some(atomic_var_type) = array_atomic_types.pop() {
        if let TAtomic::GenericParameter(parameter) = atomic_var_type {
            array_atomic_types.extend(&parameter.constraint.types);

            continue;
        }

        match atomic_var_type {
            TAtomic::Array(TArray::List(_)) => {
                let new_type = handle_array_access_on_list(
                    context,
                    block_context,
                    access_span,
                    atomic_var_type,
                    index_type,
                    in_assignment,
                    &mut has_valid_expected_index,
                );

                if let Some(existing_type) = value_type {
                    value_type =
                        Some(add_union_type(existing_type, &new_type, context.codebase, context.interner, false));
                } else {
                    value_type = Some(new_type);
                }
            }
            TAtomic::Array(TArray::Keyed(_)) => {
                let mut possibly_undefined = false;
                let mut new_type = handle_array_access_on_keyed_array(
                    context,
                    block_context,
                    access_index_span,
                    atomic_var_type,
                    index_type,
                    in_assignment,
                    &mut has_valid_expected_index,
                    block_context.inside_isset || block_context.inside_unset,
                    &mut possibly_undefined,
                    &mut false,
                );

                new_type.set_possibly_undefined(possibly_undefined, None);

                if let Some(existing_type) = value_type {
                    value_type =
                        Some(add_union_type(existing_type, &new_type, context.codebase, context.interner, false));
                } else {
                    value_type = Some(new_type);
                }
            }
            TAtomic::Scalar(TScalar::String(_)) => {
                let new_type = handle_array_access_on_string(
                    context,
                    atomic_var_type.clone(),
                    index_type.clone(),
                    &mut Vec::new(),
                    &mut has_valid_expected_index,
                );

                if let Some(existing_type) = value_type {
                    value_type =
                        Some(add_union_type(existing_type, &new_type, context.codebase, context.interner, false));
                } else {
                    value_type = Some(new_type);
                }
            }
            TAtomic::Mixed(mixed) if mixed.could_be_truthy_or_non_null() => {
                let new_type = handle_array_access_on_mixed(
                    context,
                    block_context,
                    artifacts,
                    access_span,
                    atomic_var_type,
                    array_type,
                    value_type.clone(),
                );

                if let Some(existing_type) = value_type {
                    value_type =
                        Some(add_union_type(existing_type, &new_type, context.codebase, context.interner, false));
                } else {
                    value_type = Some(new_type);
                }

                has_valid_expected_index = true;
            }
            TAtomic::Never => {
                let new_type = handle_array_access_on_mixed(
                    context,
                    block_context,
                    artifacts,
                    access_span,
                    atomic_var_type,
                    array_type,
                    value_type.clone(),
                );

                if let Some(existing_type) = value_type {
                    value_type =
                        Some(add_union_type(existing_type, &new_type, context.codebase, context.interner, false));
                } else {
                    value_type = Some(new_type);
                }

                has_valid_expected_index = true;
            }
            TAtomic::Null => {
                if !in_assignment {
                    if !block_context.inside_isset {
                        context.buffer.report(
                            TypingIssueKind::PossiblyNullArrayAccess,
                            Issue::error("Cannot perform array access on `null`.")
                            .with_annotation(Annotation::primary(access_array_span).with_message("The expression is `null` here."))
                            .with_note("Attempting to read or write an array index on `null` will result in a runtime error.")
                            .with_help("Ensure the variable holds an array before accessing it, possibly by checking with `is_array()` or initializing it."),
                        );
                    }

                    value_type = Some(add_optional_union_type(
                        get_null(),
                        value_type.as_ref(),
                        context.codebase,
                        context.interner,
                    ));
                }

                has_valid_expected_index = true;
            }
            TAtomic::Object(TObject::Named(_named_object)) => {
                // TODO(azjezz): handle ArrayAccess on objects
            }
            _ => {
                has_valid_expected_index = true;
            }
        }
    }

    if !has_valid_expected_index {
        let index_type_str = index_type.get_id(Some(context.interner));
        let array_type_str = array_type.get_id(Some(context.interner));

        let mut mixed_with_any = false;
        if index_type.is_mixed_with_any(&mut mixed_with_any) {
            for origin in &index_type.parent_nodes {
                artifacts.data_flow_graph.add_mixed_data(origin, access_span);
            }

            context.buffer.report(
                if mixed_with_any { TypingIssueKind::MixedAnyArrayIndex } else { TypingIssueKind::MixedArrayIndex },
                Issue::error(format!(
                    "Invalid array index type `{index_type_str}` used for array access on `{array_type_str}`."
                ))
                .with_annotation(
                    Annotation::primary(access_index_span)
                        .with_message(format!("Index type `{index_type_str}` is not guaranteed to be `int` or `string`."))
                )
                .with_note(
                    "Array indices must be `integer`s or `string`s."
                )
                .with_help(
                    "Ensure the index expression evaluates to an `integer` or `string`, potentially using type checks or assertions."
                ),
            );
        } else if index_type.has_array_key_like() && array_type.is_array() {
            context.buffer.report(
                TypingIssueKind::MismatchedArrayIndex,
                Issue::error(format!(
                    "Invalid array key type: `{index_type_str}` is not a valid key for this array."
                ))
                .with_annotation(
                    Annotation::primary(access_index_span)
                        .with_message(format!("This key has type `{index_type_str}`..."))
                )
                .with_annotation(
                    Annotation::secondary(access_array_span)
                        .with_message(format!("...but this array (type `{array_type_str}` ) has a more specific key type."))
                )
                .with_note(
                    "While the provided key is a valid array key type in general (an `int` or `string`), it is not compatible with the specific key type expected by this array."
                )
                .with_help(
                    "Check the array's definition (e.g., in a docblock) to see what key type it expects. It might expect only `int` keys for a list, or specific `string` keys for a shape."
                ),
            );
        } else {
            context.buffer.report(
                TypingIssueKind::InvalidArrayIndex,
                Issue::error(format!(
                    "Invalid array index type `{index_type_str}` used for array access on `{array_type_str}`."
                ))
                .with_annotation(
                    Annotation::primary(access_index_span)
                        .with_message(format!("Type `{index_type_str}` cannot be used as an array index.")),
                )
                .with_note("Array indices must be `integer`s or `string`s.")
                .with_help("Ensure the index expression evaluates to an `integer` or `string`."),
            );
        }
    }

    let array_access_type = value_type;
    if let Some(array_access_type) = array_access_type {
        array_access_type
    } else {
        // shouldn’t happen, but don’t crash
        get_mixed_any()
    }
}

// Handle array access on vec-list collections
pub(crate) fn handle_array_access_on_list(
    context: &mut Context<'_>,
    block_context: &mut BlockContext,
    span: Span,
    list: &TAtomic,
    dim_type: &TUnion,
    in_assignment: bool,
    has_valid_expected_index: &mut bool,
) -> TUnion {
    let mut union_comparison_result = ComparisonResult::new();
    let index_type_contained_by_expected = is_contained_by(
        context.codebase,
        context.interner,
        dim_type,
        &get_int(),
        false,
        false,
        false,
        &mut union_comparison_result,
    );

    if index_type_contained_by_expected {
        *has_valid_expected_index = true;
    }

    if let TAtomic::Array(TArray::List(TList { known_elements: Some(known_elements), element_type, .. })) = list {
        let mut type_param = Cow::Borrowed(element_type.as_ref());
        if let Some(val) = dim_type.get_single_literal_int_value() {
            let index = val as usize;

            if let Some((actual_possibly_undefined, actual_value)) = known_elements.get(&index) {
                *has_valid_expected_index = true;

                let mut resulting_type = actual_value.clone();
                if *actual_possibly_undefined {
                    resulting_type.set_possibly_undefined(true, None);

                    if !block_context.inside_isset && !block_context.inside_unset && !in_assignment {
                        // oh no!
                        context.buffer.report(
                        TypingIssueKind::PossiblyUndefinedIntArrayIndex,
                        Issue::warning(format!(
                            "Possibly undefined array key `{}` accessed on `{}`.",
                            val,
                            list.get_id(Some(context.interner))
                        ))
                        .with_annotation(
                            Annotation::primary(span)
                                .with_message(format!("Key `{val}` might not exist."))
                        )
                        .with_note(
                            "The analysis indicates this specific integer key might not be set when this access occurs."
                        )
                        .with_help(
                            format!(
                                "Ensure the key `{val}` is always set before accessing it, or use `isset()` or the null coalesce operator (`??`) to handle potential missing keys."
                            )
                        ),
                    );
                    }
                }

                return resulting_type;
            }

            if !in_assignment {
                if type_param.is_never() {
                    context.buffer.report(
                        TypingIssueKind::UndefinedIntArrayIndex,
                        Issue::error(format!(
                            "Undefined list index `{}` accessed on `{}`.",
                            index,
                            list.get_id(Some(context.interner))
                        ))
                        .with_annotation(
                            Annotation::primary(span)
                                .with_message(format!("Key `{index}` does not exist."))
                        )
                        .with_note(
                            "The analysis determined that this integer index is outside the defined bounds or known keys of the list."
                        )
                        .with_help(
                            format!(
                                "Ensure the index `{index}` exists before accessing it, or adjust the list access logic."
                            )
                        ),
                    );

                    return get_null();
                }

                let mut resulting_type = type_param.into_owned();
                resulting_type.set_possibly_undefined(true, None);

                return resulting_type;
            }
        }

        for (_, known_item) in known_elements.values() {
            type_param = Cow::Owned(add_union_type(
                type_param.into_owned(),
                known_item,
                context.codebase,
                context.interner,
                false,
            ));
        }

        return if type_param.is_never() { get_mixed() } else { type_param.into_owned() };
    } else if let TAtomic::Array(TArray::List(TList { element_type, .. })) = list {
        return if element_type.is_never() {
            if !in_assignment && !block_context.inside_isset && !block_context.inside_unset {
                context.buffer.report(
                    TypingIssueKind::ImpossibleArrayAccess,
                    Issue::error(format!(
                        "Cannot access elements of an empty list `{}`.",
                        list.get_id(Some(context.interner))
                    ))
                    .with_annotation(
                        Annotation::primary(span).with_message("The list is empty, no elements to access."),
                    )
                    .with_note("Attempting to access an element in an empty list will always result in a `null` value.")
                    .with_help("Ensure the list is not empty before accessing its elements."),
                );
            }

            get_null()
        } else {
            let mut element_type = *element_type.clone();
            element_type.set_possibly_undefined(true, None);

            element_type
        };
    }

    get_mixed()
}

// Handle array access on dict-like collections
pub(crate) fn handle_array_access_on_keyed_array(
    context: &mut Context<'_>,
    block_context: &mut BlockContext,
    span: Span,
    keyed_array: &TAtomic,
    index_type: &TUnion,
    in_assignment: bool,
    has_valid_expected_index: &mut bool,
    allow_possibly_undefined: bool,
    has_possibly_undefined: &mut bool,
    has_matching_dict_key: &mut bool,
) -> TUnion {
    let TAtomic::Array(TArray::Keyed(keyed_array)) = keyed_array else {
        return get_never();
    };

    let key_parameter = if in_assignment || block_context.inside_isset {
        Cow::Owned(get_arraykey())
    } else if let Some(parameters) = keyed_array.get_generic_parameters() {
        Cow::Borrowed(parameters.0)
    } else {
        Cow::Owned(get_never())
    };

    let mut has_value_parameter = false;
    let mut value_parameter = if let Some(parameters) = keyed_array.get_generic_parameters() {
        has_value_parameter = true;

        Cow::Borrowed(parameters.1)
    } else {
        Cow::Owned(get_never())
    };

    let mut union_comparison_result = ComparisonResult::new();
    let index_type_contained_by_expected = is_contained_by(
        context.codebase,
        context.interner,
        index_type,
        &key_parameter,
        false,
        false,
        false,
        &mut union_comparison_result,
    );

    if index_type_contained_by_expected {
        *has_valid_expected_index = true;
    }

    if let Some(known_items) = keyed_array.get_known_items() {
        if let Some(array_key) = index_type.get_single_array_key() {
            let possible_value = known_items.get(&array_key).cloned();
            if let Some((actual_possibly_undefined, actual_value)) = possible_value {
                *has_valid_expected_index = true;
                *has_matching_dict_key = true;

                let expression_type = actual_value;
                if actual_possibly_undefined {
                    *has_possibly_undefined = true;
                    if !in_assignment && !allow_possibly_undefined {
                        context.buffer.report(
                            match &array_key {
                                ArrayKey::Integer(_) => TypingIssueKind::PossiblyUndefinedIntArrayIndex,
                                _ => TypingIssueKind::PossiblyUndefinedStringArrayIndex,
                            },
                            Issue::warning(format!(
                                "Possibly undefined array key {} accessed on `{}`.",
                                array_key,
                                keyed_array.get_id(Some(context.interner))
                            ))
                            .with_annotation(
                                Annotation::primary(span)
                                    .with_message(format!("Key {array_key} might not exist."))
                            )
                            .with_note(
                                "The analysis indicates this specific key might not be set when this access occurs."
                            )
                            .with_help(
                                format!(
                                    "Ensure the key {array_key} is always set before accessing it, or use `isset()` or the null coalesce operator (`??`) to handle potential missing keys."
                                )
                            ),
                        );
                    }
                }

                return expression_type;
            }

            if !in_assignment {
                if has_value_parameter {
                    *has_possibly_undefined = true;

                    return value_parameter.into_owned();
                }

                let result = if !block_context.inside_isset {
                    context.buffer.report(
                        TypingIssueKind::UndefinedStringArrayIndex,
                        Issue::error(format!(
                            "Undefined array key {} accessed on `{}`.",
                            array_key,
                            keyed_array.get_id(Some(context.interner))
                        ))
                        .with_annotation(
                            Annotation::primary(span)
                                .with_message(format!("Key {array_key} does not exist."))
                        )
                        .with_note(
                            "Attempting to access a non-existent string key will raise a warning/notice at runtime."
                        )
                        .with_help(
                            format!(
                                "Ensure the key {array_key} exists before accessing it, or use `isset()` or the null coalesce operator (`??`) to handle potential missing keys."
                            )
                        ),
                    );

                    if has_value_parameter { get_mixed() } else { get_null() }
                } else {
                    context.buffer.report(
                        TypingIssueKind::ImpossibleNonnullEntryCheck,
                        Issue::warning(format!(
                            "Impossible `isset` check on key `{}` accessed on `{}`.",
                            array_key,
                            keyed_array.get_id(Some(context.interner))
                        ))
                        .with_annotation(
                            Annotation::primary(span)
                                .with_message(format!("`isset` on key `{array_key}` will always be false here."))
                        )
                        .with_note(
                            format!(
                                "The analysis determined that the key `{array_key}` definitely does not exist in this array, so checking `isset` is unnecessary."
                            )
                        )
                        .with_help(
                            "Remove the redundant `isset` check."
                        ),
                    );

                    get_mixed()
                };

                // since we're emitting a very specific error
                // we don't want to emit another error afterwards
                *has_valid_expected_index = true;

                return result;
            }
        }

        for (_, known_item) in known_items.values() {
            value_parameter = Cow::Owned(add_union_type(
                value_parameter.into_owned(),
                known_item,
                context.codebase,
                context.interner,
                false,
            ));
        }

        let array_key = get_arraykey();
        let is_contained = is_contained_by(
            context.codebase,
            context.interner,
            &key_parameter,
            if index_type.is_mixed() { &array_key } else { index_type },
            true,
            value_parameter.ignore_falsable_issues,
            false,
            &mut ComparisonResult::new(),
        );

        if is_contained {
            *has_valid_expected_index = true;
        }

        *has_possibly_undefined = true;

        value_parameter.into_owned()
    } else {
        // TODO Handle Assignments
        // if (block_context.inside_assignment && replacement_type) {

        // }
        if has_value_parameter {
            if !in_assignment {
                *has_possibly_undefined = true;

                if !allow_possibly_undefined && index_type.get_single_array_key().is_some() {
                    let index_type_str = index_type.get_id(Some(context.interner));

                    context.buffer.report(
                        TypingIssueKind::PossiblyUndefinedArrayIndex,
                        Issue::warning(format!(
                            "Possibly undefined array key `{index_type_str}` accessed on `{}`.",
                            keyed_array.get_id(Some(context.interner))
                        ))
                        .with_annotation(
                            Annotation::primary(span)
                                .with_message(format!("Key `{index_type_str}` might not exist."))
                        )
                        .with_note(
                            "The analysis indicates this specific key might not be set when this access occurs."
                        )
                        .with_help(
                            format!(
                                "Ensure the key {index_type_str} is always set before accessing it, or use `isset()` or the null coalesce operator (`??`) to handle potential missing keys."
                            )
                        ),
                    );
                }
            }

            value_parameter.into_owned()
        } else if block_context.inside_assignment {
            get_never()
        } else {
            get_null()
        }
    }
}

// Handle array access on strings
pub(crate) fn handle_array_access_on_string(
    context: &mut Context<'_>,
    string: TAtomic,
    index_type: TUnion,
    expected_index_types: &mut Vec<String>,
    has_valid_expected_index: &mut bool,
) -> TUnion {
    let mut non_empty = false;

    let valid_index_type = if let TAtomic::Scalar(TScalar::String(scalar_string)) = string {
        non_empty = scalar_string.is_non_empty();

        if let Some(TStringLiteral::Value(val)) = scalar_string.literal {
            if val.is_empty() {
                get_never()
            } else {
                TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, val.len() as i64 - 1)))])
            }
        } else {
            get_int()
        }
    } else {
        get_int()
    };

    if !is_contained_by(
        context.codebase,
        context.interner,
        &index_type,
        &valid_index_type,
        false,
        false,
        false,
        &mut ComparisonResult::new(),
    ) {
        expected_index_types.push(valid_index_type.get_id(Some(context.interner)));
    } else {
        *has_valid_expected_index = true;
    }

    if non_empty { get_non_empty_string() } else { get_string() }
}

pub(crate) fn handle_array_access_on_mixed(
    context: &mut Context<'_>,
    block_context: &mut BlockContext,
    artifacts: &mut AnalysisArtifacts,
    span: Span,
    mixed: &TAtomic,
    mixed_union: &TUnion,
    stmt_type: Option<TUnion>,
) -> TUnion {
    if !block_context.inside_isset {
        for origin in &mixed_union.parent_nodes {
            artifacts.data_flow_graph.add_mixed_data(origin, span);
        }

        if block_context.inside_assignment {
            if mixed.is_any() {
                context.buffer.report(
                    TypingIssueKind::MixedAnyArrayAssignment,
                    Issue::error(format!(
                        "Unsafe array assignment on type `{}`.",
                        mixed.get_id(Some(context.interner))
                    ))
                    .with_annotation(
                        Annotation::primary(span)
                            .with_message("Cannot safely assign to index because base type is `any`.")
                    )
                    .with_note(
                        "The variable being assigned to might not be an array at runtime."
                    )
                    .with_help(
                        "Ensure the variable holds an array before assigning to an index, potentially using type checks or assertions."
                    ),
                );
            } else if let TAtomic::Never = mixed {
                context.buffer.report(
                    TypingIssueKind::ImpossibleArrayAssignment,
                    Issue::error(
                        "Cannot perform array assignment on type `never`."
                    )
                    .with_annotation(
                        Annotation::primary(span)
                            .with_message("Base expression has type `never`.")
                    )
                    .with_note(
                        "An expression with type `never` cannot produce a value to assign to."
                    )
                    .with_help(
                        "This code path is unreachable because the base expression will never complete normally (e.g., it throws, exits, or loops forever). Remove the assignment."
                    ),
                );
            } else {
                context.buffer.report(
                    TypingIssueKind::MixedArrayAssignment,
                    Issue::error(format!(
                        "Unsafe array assignment on type `{}`.",
                        mixed.get_id(Some(context.interner))
                    ))
                    .with_annotation(
                        Annotation::primary(span)
                            .with_message("Cannot safely assign to index because base type is `mixed`.")
                    )
                    .with_note(
                        "The variable being assigned to might not be an array at runtime."
                    )
                    .with_help(
                        "Ensure the variable holds an array before assigning to an index, potentially using type checks or assertions."
                    ),
                );
            }
        } else if mixed.is_any() {
            context.buffer.report(
                TypingIssueKind::MixedAnyArrayAccess,
                Issue::error(format!(
                    "Unsafe array access on type `{}`.",
                    mixed.get_id(None)
                ))
                .with_annotation(
                    Annotation::primary(span)
                        .with_message("Cannot safely access index because base type is `any`.")
                )
                .with_note(
                    "The variable being accessed might not be an array at runtime."
                )
                .with_help(
                    "Ensure the variable holds an array before accessing an index, potentially using type checks or assertions."
                ),
            );
        } else {
            context.buffer.report(
                TypingIssueKind::MixedArrayAccess,
                Issue::error(format!("Unsafe array access on type `{}`.", mixed.get_id(None)))
                .with_annotation(Annotation::primary(span).with_message("Cannot safely access index because base type is `mixed`."))
                .with_note("The variable being accessed might not be an array at runtime.")
                .with_help("Ensure the variable holds an array before accessing an index, potentially using type checks or assertions."),
            );
        }
    }

    if let Some(stmt_var_type) = artifacts.expression_types.get(&get_expression_range(&span))
        && !stmt_var_type.parent_nodes.is_empty()
    {
        let new_parent_node = DataFlowNode::get_for_local_string("mixed-var-array-access".to_string(), span);
        artifacts.data_flow_graph.add_node(new_parent_node.clone());

        for parent_node in stmt_var_type.parent_nodes.iter() {
            artifacts.data_flow_graph.add_path(parent_node, &new_parent_node, PathKind::Default);
        }
        if let Some(stmt_type) = stmt_type {
            let mut stmt_type_new = stmt_type.clone();
            stmt_type_new.parent_nodes = vec![new_parent_node.clone()];
        }
    }

    if let TAtomic::Never = mixed {
        return get_mixed_maybe_from_loop(true);
    }

    get_mixed_any()
}
