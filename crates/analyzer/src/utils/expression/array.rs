use std::borrow::Cow;

use mago_atom::atom;
use mago_codex::get_class_like;
use mago_codex::is_instance_of;
use mago_codex::metadata::class_like::ClassLikeMetadata;
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
use mago_codex::ttype::get_mixed_maybe_from_loop;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::get_null;
use mago_codex::ttype::get_specialized_template_type;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ArrayTarget<'ast, 'arena> {
    Access(&'ast ArrayAccess<'arena>),
    Append(&'ast ArrayAppend<'arena>),
}

impl<'ast, 'arena> ArrayTarget<'ast, 'arena> {
    #[inline]
    pub const fn get_array(&self) -> &'ast Expression<'arena> {
        match self {
            ArrayTarget::Access(array_access) => array_access.array,
            ArrayTarget::Append(array_append) => array_append.array,
        }
    }

    #[inline]
    pub const fn get_index(&self) -> Option<&'ast Expression<'arena>> {
        match self {
            ArrayTarget::Access(array_access) => Some(array_access.index),
            ArrayTarget::Append(_) => None,
        }
    }
}

impl HasSpan for ArrayTarget<'_, '_> {
    fn span(&self) -> Span {
        match self {
            ArrayTarget::Access(array_access) => array_access.span(),
            ArrayTarget::Append(array_append) => array_append.span(),
        }
    }
}

impl<'ast, 'arena> From<&'ast ArrayAccess<'arena>> for ArrayTarget<'ast, 'arena> {
    fn from(array_access: &'ast ArrayAccess<'arena>) -> Self {
        ArrayTarget::Access(array_access)
    }
}

impl<'ast, 'arena> From<&'ast ArrayAppend<'arena>> for ArrayTarget<'ast, 'arena> {
    fn from(array_append: &'ast ArrayAppend<'arena>) -> Self {
        ArrayTarget::Append(array_append)
    }
}

pub(crate) fn get_array_target_type_given_index<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    access_span: Span,
    access_array_span: Span,
    access_index_span: Option<Span>,
    array_like_type: &TUnion,
    index_type: &TUnion,
    in_assignment: bool,
    extended_var_id: &Option<String>,
) -> TUnion {
    let mut has_valid_expected_index = false;

    let access_index_span = match access_index_span {
        Some(index) => index,
        None => access_span,
    };

    if array_like_type.is_never() || index_type.is_never() {
        return get_never();
    }

    if index_type.is_null() {
        context.collector.report_with_code(
            IssueCode::NullArrayIndex,
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

    if index_type.is_nullable() && !block_context.inside_isset && !index_type.ignore_nullable_issues {
        context.collector.report_with_code(
            IssueCode::PossiblyNullArrayIndex,
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

    let mut array_atomic_types = array_like_type.types.iter().collect::<Vec<_>>();

    let mut value_type = None;
    let mut expected_index_types = vec![];
    while let Some(atomic_var_type) = array_atomic_types.pop() {
        if let TAtomic::GenericParameter(parameter) = atomic_var_type {
            array_atomic_types.extend(parameter.constraint.types.as_ref());

            continue;
        }

        match atomic_var_type {
            TAtomic::Array(TArray::List(_)) => {
                let new_type = handle_array_access_on_list(
                    context,
                    block_context,
                    Some(access_span),
                    atomic_var_type,
                    index_type,
                    in_assignment,
                    &mut has_valid_expected_index,
                    &mut expected_index_types,
                );

                if let Some(existing_type) = value_type {
                    value_type = Some(add_union_type(existing_type, &new_type, context.codebase, false));
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
                    context.settings.allow_possibly_undefined_array_keys
                        || block_context.inside_isset
                        || block_context.inside_unset,
                    &mut possibly_undefined,
                    &mut false,
                    &mut expected_index_types,
                );

                new_type.set_possibly_undefined(possibly_undefined, None);

                if let Some(existing_type) = value_type {
                    value_type = Some(add_union_type(existing_type, &new_type, context.codebase, false));
                } else {
                    value_type = Some(new_type);
                }
            }
            TAtomic::Scalar(TScalar::String(_)) => {
                let new_type = handle_array_access_on_string(
                    context,
                    atomic_var_type.clone(),
                    index_type.clone(),
                    &mut has_valid_expected_index,
                    &mut expected_index_types,
                );

                if let Some(existing_type) = value_type {
                    value_type = Some(add_union_type(existing_type, &new_type, context.codebase, false));
                } else {
                    value_type = Some(new_type);
                }
            }
            TAtomic::Mixed(mixed) if mixed.could_be_truthy_or_non_null() => {
                let new_type = handle_array_access_on_mixed(context, block_context, access_span, atomic_var_type);

                if let Some(existing_type) = value_type {
                    value_type = Some(add_union_type(existing_type, &new_type, context.codebase, false));
                } else {
                    value_type = Some(new_type);
                }

                has_valid_expected_index = true;
            }
            TAtomic::Never => {
                let new_type = handle_array_access_on_mixed(context, block_context, access_span, atomic_var_type);

                if let Some(existing_type) = value_type {
                    value_type = Some(add_union_type(existing_type, &new_type, context.codebase, false));
                } else {
                    value_type = Some(new_type);
                }

                has_valid_expected_index = true;
            }
            TAtomic::Null => {
                if !array_like_type.ignore_nullable_issues && !in_assignment {
                    if !block_context.inside_isset {
                        context.collector.report_with_code(
                            IssueCode::PossiblyNullArrayAccess,
                            Issue::error("Cannot perform array access on `null`.")
                            .with_annotation(Annotation::primary(access_array_span).with_message("The expression is `null` here."))
                            .with_note("Attempting to read or write an array index on `null` will result in a runtime error.")
                            .with_help("Ensure the variable holds an array before accessing it, possibly by checking with `is_array()` or initializing it."),
                        );
                    }

                    value_type = Some(add_optional_union_type(get_null(), value_type.as_ref(), context.codebase));
                }

                has_valid_expected_index = true;
            }
            TAtomic::Object(TObject::Named(_)) => {
                let new_type = handle_array_access_on_named_object(
                    context,
                    access_span,
                    atomic_var_type,
                    index_type,
                    &mut has_valid_expected_index,
                    &mut expected_index_types,
                );

                value_type = Some(add_optional_union_type(new_type, value_type.as_ref(), context.codebase));
            }
            _ => {
                has_valid_expected_index = true;
            }
        }
    }

    if !has_valid_expected_index {
        let index_type_str = index_type.get_id();
        let array_like_type_str = array_like_type.get_id();
        let expected_index_types_str = expected_index_types
            .iter()
            .flat_map(|union| union.types.as_ref())
            .map(|t| t.get_id().as_str())
            .collect::<Vec<_>>();

        let expected_types_list = if let Some(last_index_str) = expected_index_types_str.last() {
            if expected_index_types_str.len() == 1 {
                format!("`{last_index_str}`")
            } else {
                let rest = &expected_index_types_str[..expected_index_types_str.len() - 1];
                format!("`{}` or `{}`", rest.join("`, `"), last_index_str)
            }
        } else {
            "an expected type".to_string()
        };

        if index_type.is_mixed() {
            let note_text = if expected_index_types_str.len() == 1 {
                format!("The index for this type must be `{expected_types_list}`.")
            } else {
                format!("The index for this type must be one of: {expected_types_list}.")
            };

            let help_text = if expected_index_types_str.len() == 1 {
                format!("Ensure the index expression evaluates to `{expected_types_list}`.")
            } else {
                format!("Ensure the index expression evaluates to one of the expected types: {expected_types_list}.")
            };

            context.collector.report_with_code(
                IssueCode::MixedArrayIndex,
                Issue::error(format!(
                    "Invalid index type `{index_type_str}` used for array access on `{array_like_type_str}`."
                ))
                .with_annotation(Annotation::primary(access_index_span).with_message(format!(
                    "This index has type `{index_type_str}` which is not guaranteed to be a valid key."
                )))
                .with_note(note_text)
                .with_help(help_text),
            );
        } else if index_type.has_array_key_like() && array_like_type.is_array() {
            context.collector.report_with_code(
                IssueCode::MismatchedArrayIndex,
                Issue::error(format!(
                    "Invalid array key type: `{index_type_str}` is not a valid key for this array."
                ))
                .with_annotation(
                    Annotation::primary(access_index_span)
                        .with_message(format!("This key has type `{index_type_str}`..."))
                )
                .with_annotation(
                    Annotation::secondary(access_array_span)
                        .with_message(format!("...but this array (type `{array_like_type_str}` ) has a more specific key type."))
                )
                .with_note(
                    "While the provided key is a valid array key type in general (an `int` or `string`), it is not compatible with the specific key type expected by this array."
                )
                .with_help(
                    "Check the array's definition (e.g., in a docblock) to see what key type it expects. It might expect only `int` keys for a list, or specific `string` keys for a shape."
                ),
            );
        } else {
            let note_text = if expected_index_types_str.len() == 1 {
                format!("The only valid index type for `{array_like_type_str}` is {expected_types_list}.")
            } else {
                format!("Valid index types for `{array_like_type_str}` are: {expected_types_list}.")
            };

            let help_text = if expected_index_types_str.len() == 1 {
                format!("Ensure the index expression evaluates to {expected_types_list}.")
            } else {
                format!("Ensure the index expression evaluates to one of the expected types: {expected_types_list}.")
            };

            context.collector.report_with_code(
                IssueCode::InvalidArrayIndex,
                Issue::error(format!(
                    "Invalid index type `{index_type_str}` used for array access on `{array_like_type_str}`."
                ))
                .with_annotation(
                    Annotation::primary(access_index_span)
                        .with_message(format!("Type `{index_type_str}` cannot be used as an index here.")),
                )
                .with_note(note_text)
                .with_help(help_text),
            );
        }
    }

    match value_type {
        Some(mut value_type) => {
            value_type.possibly_undefined |= array_like_type.possibly_undefined;
            value_type.possibly_undefined_from_try |= array_like_type.possibly_undefined_from_try;
            value_type.ignore_falsable_issues |= array_like_type.ignore_falsable_issues;
            value_type.ignore_falsable_issues |= array_like_type.ignore_falsable_issues;

            value_type
        }
        None => get_mixed(),
    }
}

pub(crate) fn handle_array_access_on_list<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    span: Option<Span>,
    list: &TAtomic,
    dim_type: &TUnion,
    in_assignment: bool,
    has_valid_expected_index: &mut bool,
    expected_index_types: &mut Vec<TUnion>,
) -> TUnion {
    let expected_key_type = get_int();

    let mut union_comparison_result = ComparisonResult::new();
    let index_type_contained_by_expected = is_contained_by(
        context.codebase,
        dim_type,
        &expected_key_type,
        true,
        false,
        false,
        &mut union_comparison_result,
    );

    if index_type_contained_by_expected {
        *has_valid_expected_index = true;
    } else {
        expected_index_types.push(expected_key_type);
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

                    if !context.settings.allow_possibly_undefined_array_keys
                        && !block_context.inside_isset
                        && !block_context.inside_unset
                        && !in_assignment
                        && let Some(span) = span
                    {
                        // oh no!
                        context.collector.report_with_code(
                            IssueCode::PossiblyUndefinedIntArrayIndex,
                            Issue::warning(format!(
                                "Possibly undefined array key `{}` accessed on `{}`.",
                                val,
                                list.get_id()
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
                if type_param.is_never()
                    && let Some(span) = span
                {
                    context.collector.report_with_code(
                        IssueCode::UndefinedIntArrayIndex,
                        Issue::error(format!(
                            "Undefined list index `{}` accessed on `{}`.",
                            index,
                            list.get_id()
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
            type_param = Cow::Owned(add_union_type(type_param.into_owned(), known_item, context.codebase, false));
        }

        return if type_param.is_never() { get_mixed() } else { type_param.into_owned() };
    } else if let TAtomic::Array(TArray::List(TList { element_type, .. })) = list {
        return if element_type.is_never() {
            if !in_assignment
                && !block_context.inside_isset
                && !block_context.inside_unset
                && let Some(span) = span
            {
                context.collector.report_with_code(
                    IssueCode::ImpossibleArrayAccess,
                    Issue::error(format!("Cannot access elements of an empty list `{}`.", list.get_id()))
                        .with_annotation(
                            Annotation::primary(span).with_message("The list is empty, no elements to access."),
                        )
                        .with_note(
                            "Attempting to access an element in an empty list will always result in a `null` value.",
                        )
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

pub(crate) fn handle_array_access_on_keyed_array<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    span: Span,
    keyed_array: &TAtomic,
    index_type: &TUnion,
    in_assignment: bool,
    has_valid_expected_index: &mut bool,
    allow_possibly_undefined: bool,
    has_possibly_undefined: &mut bool,
    has_matching_dict_key: &mut bool,
    expected_index_types: &mut Vec<TUnion>,
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
    let index_type_contained_by_expected =
        is_contained_by(context.codebase, index_type, &key_parameter, true, false, false, &mut union_comparison_result);

    if index_type_contained_by_expected {
        *has_valid_expected_index = true;
    } else {
        expected_index_types.push(key_parameter.clone().into_owned());
    }

    if let Some(known_items) = keyed_array.get_known_items() {
        if let Some(array_key) = index_type.get_single_array_key() {
            if let Some((actual_possibly_undefined, actual_value)) = known_items.get(&array_key).cloned() {
                *has_valid_expected_index = true;
                *has_matching_dict_key = true;

                let expression_type = actual_value;
                if actual_possibly_undefined {
                    *has_possibly_undefined = true;
                    if !in_assignment && !allow_possibly_undefined {
                        context.collector.report_with_code(
                            match &array_key {
                                ArrayKey::Integer(_) => IssueCode::PossiblyUndefinedIntArrayIndex,
                                _ => IssueCode::PossiblyUndefinedStringArrayIndex,
                            },
                            Issue::warning(format!(
                                "Possibly undefined array key {} accessed on `{}`.",
                                array_key,
                                keyed_array.get_id()
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
            } else {
                if in_assignment && !has_value_parameter {
                    // In an assignment to a non-existent key, the value before assignment is effectively null.
                    // This allows upstream logic to promote it to an array.
                    return get_null();
                }

                // This is a read access to a non-existent key.
                if context.settings.allow_possibly_undefined_array_keys && has_value_parameter {
                    *has_possibly_undefined = true;

                    return value_parameter.into_owned();
                }

                let result = if !block_context.inside_isset {
                    context.collector.report_with_code(
                        IssueCode::UndefinedStringArrayIndex,
                        Issue::error(format!(
                            "Undefined array key {} accessed on `{}`.",
                            array_key,
                            keyed_array.get_id()
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
                    context.collector.report_with_code(
                        IssueCode::ImpossibleNonnullEntryCheck,
                        Issue::warning(format!(
                            "Impossible `isset` check on key `{}` accessed on `{}`.",
                            array_key,
                            keyed_array.get_id()
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
            value_parameter =
                Cow::Owned(add_union_type(value_parameter.into_owned(), known_item, context.codebase, false));
        }

        let array_key = get_arraykey();
        let is_contained = is_contained_by(
            context.codebase,
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
                    let index_type_str = index_type.get_id();

                    context.collector.report_with_code(
                        IssueCode::PossiblyUndefinedArrayIndex,
                        Issue::warning(format!(
                            "Possibly undefined array key `{index_type_str}` accessed on `{}`.",
                            keyed_array.get_id()
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
        } else if in_assignment {
            get_never()
        } else {
            get_null()
        }
    }
}

pub(crate) fn handle_array_access_on_named_object<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    span: Span,
    named_object: &TAtomic,
    index_type: &TUnion,
    has_valid_expected_index: &mut bool,
    expected_index_types: &mut Vec<TUnion>,
) -> TUnion {
    fn get_array_access_classes<'ctx, 'arena>(
        context: &mut Context<'ctx, 'arena>,
        atomic: &TAtomic,
    ) -> Option<(Vec<&'ctx ClassLikeMetadata>, TUnion, TUnion)> {
        let mut parameters = vec![];
        let metadata = 'metadata: {
            let TAtomic::Object(TObject::Named(named_object)) = atomic else {
                break 'metadata None;
            };

            let array_access = atom("ArrayAccess");
            if !is_instance_of(context.codebase, &named_object.name, &array_access) {
                break 'metadata None;
            }

            let Some(metadata) = get_class_like(context.codebase, &named_object.name) else {
                break 'metadata None;
            };

            let Some(array_access_metadata) = get_class_like(context.codebase, &array_access) else {
                break 'metadata None;
            };

            let Some(key_template_name) = array_access_metadata.template_types.first().map(|(name, _)| name) else {
                break 'metadata None;
            };

            let Some(value_template_name) = array_access_metadata.template_types.get(1).map(|(name, _)| name) else {
                break 'metadata None;
            };

            let key_type = get_specialized_template_type(
                context.codebase,
                key_template_name,
                &array_access,
                metadata,
                named_object.get_type_parameters(),
            )
            .unwrap_or_else(get_mixed);

            let value_type = get_specialized_template_type(
                context.codebase,
                value_template_name,
                &array_access,
                metadata,
                named_object.get_type_parameters(),
            )
            .unwrap_or_else(get_mixed);

            parameters.push((key_type, value_type));

            Some(metadata)
        };

        let mut class_likes = vec![];

        if let Some(metadata) = metadata {
            class_likes.push(metadata);
        }

        if let Some(intersection_types) = atomic.get_intersection_types() {
            for intersection_type in intersection_types {
                if let Some(intersections) = get_array_access_classes(context, intersection_type) {
                    class_likes.extend(intersections.0);

                    parameters.push((intersections.1, intersections.2));
                }
            }
        }

        let mut key_type = None;
        let mut value_type = None;

        for (key_parameter_type, value_parameter_type) in parameters {
            key_type = Some(add_optional_union_type(key_parameter_type, key_type.as_ref(), context.codebase));

            value_type = Some(add_optional_union_type(value_parameter_type, value_type.as_ref(), context.codebase));
        }

        if class_likes.is_empty() {
            return None;
        }

        Some((class_likes, key_type.unwrap_or_else(get_mixed), value_type.unwrap_or_else(get_mixed)))
    }

    // TODO: we should analyze calls to `offsetSet` and `offsetGet` here.
    let Some((_array_access_classes, expected_key_type, mut resulting_value_type)) =
        get_array_access_classes(context, named_object)
    else {
        context.collector.report_with_code(
            IssueCode::InvalidArrayAccess,
            Issue::error(format!(
                "Cannot access array index on object `{}` that does not implement `ArrayAccess`.",
                named_object.get_id()
            ))
            .with_annotation(Annotation::primary(span).with_message("Object does not implement `ArrayAccess`."))
            .with_note("Only objects implementing `ArrayAccess` can be accessed like arrays.")
            .with_help("Ensure the object implements `ArrayAccess` before attempting to access it as an array."),
        );

        return get_never();
    };

    let mut union_comparison_result = ComparisonResult::new();
    let index_type_contained_by_expected = is_contained_by(
        context.codebase,
        index_type,
        &expected_key_type,
        false,
        false,
        false,
        &mut union_comparison_result,
    );

    if index_type_contained_by_expected {
        *has_valid_expected_index = true;
    } else {
        expected_index_types.push(expected_key_type);
    }

    resulting_value_type.possibly_undefined = true;
    resulting_value_type
}

pub(crate) fn handle_array_access_on_string<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    string: TAtomic,
    index_type: TUnion,
    has_valid_expected_index: &mut bool,
    expected_index_types: &mut Vec<TUnion>,
) -> TUnion {
    let mut non_empty = false;

    let valid_index_type = if let TAtomic::Scalar(TScalar::String(scalar_string)) = string {
        non_empty = scalar_string.is_non_empty();

        if let Some(TStringLiteral::Value(val)) = scalar_string.literal {
            if val.is_empty() {
                get_never()
            } else {
                TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, val.len() as i64 - 1))))
            }
        } else {
            get_int()
        }
    } else {
        get_int()
    };

    if !is_contained_by(
        context.codebase,
        &index_type,
        &valid_index_type,
        false,
        false,
        false,
        &mut ComparisonResult::new(),
    ) {
        expected_index_types.push(valid_index_type);
    } else {
        *has_valid_expected_index = true;
    }

    if non_empty { get_non_empty_string() } else { get_string() }
}

pub(crate) fn handle_array_access_on_mixed<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    span: Span,
    mixed: &TAtomic,
) -> TUnion {
    if !block_context.inside_isset {
        if block_context.inside_assignment {
            if let TAtomic::Never = mixed {
                context.collector.report_with_code(
                    IssueCode::ImpossibleArrayAssignment,
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
                context.collector.report_with_code(
                    IssueCode::MixedArrayAssignment,
                    Issue::error(format!(
                        "Unsafe array assignment on type `{}`.",
                        mixed.get_id()
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
        } else {
            context.collector.report_with_code(
                IssueCode::MixedArrayAccess,
                Issue::error(format!("Unsafe array access on type `{}`.", mixed.get_id()))
                .with_annotation(Annotation::primary(span).with_message("Cannot safely access index because base type is `mixed`."))
                .with_note("The variable being accessed might not be an array at runtime.")
                .with_help("Ensure the variable holds an array before accessing an index, potentially using type checks or assertions."),
            );
        }
    }

    if let TAtomic::Never = mixed {
        return get_mixed_maybe_from_loop(true);
    }

    get_mixed()
}
