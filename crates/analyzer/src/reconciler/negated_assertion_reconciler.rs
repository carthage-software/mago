use std::borrow::Cow;

use mago_atom::Atom;
use mago_atom::AtomSet;
use mago_codex::assertion::Assertion;
use mago_codex::consts::MAX_ENUM_CASES_FOR_ANALYSIS;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::r#enum::TEnum;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::combiner;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::atomic_comparator;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_placeholder;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_span::Span;

use crate::reconciler::Context;
use crate::reconciler::assertion_reconciler::intersect_atomic_with_atomic;
use crate::reconciler::simple_negated_assertion_reconciler;
use crate::reconciler::trigger_issue_for_impossible;

pub(crate) fn reconcile(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    old_var_type_atom: Atom,
    span: Option<&Span>,
    negated: bool,
) -> TUnion {
    let is_equality = assertion.has_equality();
    if is_equality && assertion.has_literal_value() {
        if existing_var_type.is_mixed() {
            return existing_var_type.clone();
        }

        return handle_literal_negated_equality(
            context,
            assertion,
            existing_var_type,
            key,
            old_var_type_atom,
            span,
            negated,
        );
    }

    let simple_negated_type =
        simple_negated_assertion_reconciler::reconcile(context, assertion, existing_var_type, key, span, negated);

    if let Some(simple_negated_type) = simple_negated_type {
        return simple_negated_type;
    }

    let mut existing_var_type = existing_var_type.clone();

    if let Some(assertion_type) = assertion.get_type() {
        if !is_equality {
            if let Some(assertion_type) = assertion.get_type() {
                let mut has_changes = false;
                subtract_complex_type(context, assertion_type, &mut existing_var_type, &mut has_changes);

                if (!has_changes || existing_var_type.is_never())
                    && let Some(key) = &key
                    && let Some(pos) = span
                {
                    trigger_issue_for_impossible(
                        context,
                        old_var_type_atom,
                        key,
                        assertion,
                        !has_changes,
                        negated,
                        pos,
                    );
                }
            }
        } else if let Some(key) = &key
            && let Some(pos) = span
            && !union_comparator::can_expression_types_be_identical(
                context.codebase,
                &existing_var_type,
                &wrap_atomic(assertion_type.clone()),
                true,
                false,
            )
        {
            trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, true, negated, pos);
        }
    }

    if existing_var_type.types.is_empty() && !is_equality {
        if let Some(key) = &key
            && let Some(pos) = span
        {
            trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, false, negated, pos);
        }

        return get_never();
    }

    existing_var_type
}

fn subtract_complex_type(
    context: &mut Context<'_, '_>,
    assertion_type: &TAtomic,
    existing_var_type: &mut TUnion,
    can_be_disjunct: &mut bool,
) {
    let mut acceptable_types = vec![];

    let existing_atomic_types = existing_var_type.types.to_mut().drain(..).collect::<Vec<_>>();

    for existing_atomic in existing_atomic_types {
        if &existing_atomic == assertion_type {
            *can_be_disjunct = true;

            continue;
        }

        if atomic_comparator::is_contained_by(
            context.codebase,
            &existing_atomic,
            assertion_type,
            true,
            &mut ComparisonResult::new(),
        ) {
            *can_be_disjunct = true;

            // don't add as acceptable
            continue;
        }

        if atomic_comparator::is_contained_by(
            context.codebase,
            assertion_type,
            &existing_atomic,
            true,
            &mut ComparisonResult::new(),
        ) {
            *can_be_disjunct = true;
        }

        match (&existing_atomic, assertion_type) {
            (
                TAtomic::Object(TObject::Named(existing_named_object)),
                TAtomic::Object(TObject::Named(assertion_named_object)),
            ) => {
                let existing_classlike_name = existing_named_object.get_name_ref();
                let assertion_classlike_name = assertion_named_object.get_name_ref();

                if let Some(class_like_metadata) = context.codebase.get_class_like(existing_classlike_name) {
                    // handle __Sealed classes, negating where possible
                    if let Some(child_classlikes) = class_like_metadata.child_class_likes.as_ref()
                        && child_classlikes.contains(assertion_classlike_name)
                    {
                        handle_negated_class(
                            context,
                            child_classlikes,
                            &existing_atomic,
                            assertion_classlike_name,
                            &mut acceptable_types,
                        );

                        *can_be_disjunct = true;

                        continue;
                    }
                }

                if (context.codebase.interface_exists(assertion_classlike_name)
                    || context.codebase.interface_exists(existing_classlike_name))
                    && assertion_classlike_name != existing_classlike_name
                {
                    *can_be_disjunct = true;
                }

                acceptable_types.push(existing_atomic);
            }
            (TAtomic::Array(first), TAtomic::Array(second)) if first.is_keyed() && second.is_keyed() => {
                *can_be_disjunct = true;
                // todo subtract assertion keyed array from existing
                acceptable_types.push(existing_atomic);
            }
            (
                TAtomic::Object(TObject::Enum(TEnum { name: existing_enum_name, case: None })),
                TAtomic::Object(TObject::Enum(TEnum { name: assertion_enum_name, case: Some(assertion_case) })),
            ) if context.codebase.is_instance_of(assertion_enum_name, existing_enum_name) => {
                *can_be_disjunct = true;

                let Some(enum_metadata) = context.codebase.get_enum(existing_enum_name) else {
                    acceptable_types.push(existing_atomic);
                    continue;
                };

                // Enum is too large, do not subtract anything
                if enum_metadata.enum_cases.len() > MAX_ENUM_CASES_FOR_ANALYSIS {
                    acceptable_types.push(existing_atomic);
                    continue;
                }

                for (enum_case, _) in &enum_metadata.enum_cases {
                    if enum_case == assertion_case {
                        continue;
                    }

                    acceptable_types.push(TAtomic::Object(TObject::Enum(TEnum {
                        name: *existing_enum_name,
                        case: Some(*enum_case),
                    })));
                }
            }
            (TAtomic::Object(TObject::Enum(_)), TAtomic::Object(TObject::Enum(_))) => {
                *can_be_disjunct = true;
                acceptable_types.push(existing_atomic);
            }
            _ => {
                acceptable_types.push(existing_atomic);
            }
        }
    }

    if acceptable_types.is_empty() {
        acceptable_types.push(TAtomic::Never);
    } else if acceptable_types.len() > 1 && *can_be_disjunct {
        acceptable_types = combiner::combine(acceptable_types, context.codebase, false);
    }

    existing_var_type.types = Cow::Owned(acceptable_types);
}

fn handle_negated_class(
    context: &mut Context<'_, '_>,
    child_classlikes: &AtomSet,
    existing_atomic: &TAtomic,
    assertion_classlike_name: &Atom,
    acceptable_types: &mut Vec<TAtomic>,
) {
    for child_classlike in child_classlikes {
        if child_classlike != assertion_classlike_name {
            let alternate_class =
                TAtomic::Object(TObject::Named(TNamedObject::new(*child_classlike).with_type_parameters(
                    if let Some(child_metadata) = context.codebase.get_class_like(child_classlike) {
                        let placeholder_params =
                            child_metadata.template_types.iter().map(|_| get_placeholder()).collect::<Vec<_>>();

                        if placeholder_params.is_empty() { None } else { Some(placeholder_params) }
                    } else {
                        None
                    },
                )));

            if let Some(acceptable_alternate_class) =
                intersect_atomic_with_atomic(context, existing_atomic, &alternate_class)
            {
                acceptable_types.push(acceptable_alternate_class);
            }
        }
    }
}

fn handle_literal_negated_equality(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    old_var_type_atom: Atom,
    span: Option<&Span>,
    negated: bool,
) -> TUnion {
    let Some(assertion_type) = assertion.get_type() else {
        return get_never();
    };

    let mut did_remove_type = false;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for existing_atomic_type in new_var_type.types.to_mut().drain(..) {
        match &existing_atomic_type {
            TAtomic::Scalar(TScalar::String(existing_string)) => {
                let existing_literal_string = existing_atomic_type.get_literal_string_value();
                let assertion_literal_string = assertion_type.get_literal_string_value();

                match (existing_literal_string, assertion_literal_string) {
                    (Some(existing_value), Some(assertion_value)) => {
                        if existing_value == assertion_value {
                            did_remove_type = true;
                        } else {
                            acceptable_types.push(existing_atomic_type);
                        }
                    }
                    (None, Some(assertion_value)) => {
                        did_remove_type = true;

                        if assertion_value.is_empty() {
                            acceptable_types.push(TAtomic::Scalar(TScalar::String(TString::general_with_props(
                                existing_string.is_numeric,
                                existing_string.is_truthy,
                                true,
                                existing_string.is_lowercase,
                            ))));
                        } else {
                            acceptable_types.push(existing_atomic_type);
                        }
                    }
                    _ => {
                        acceptable_types.push(existing_atomic_type);
                    }
                }
            }
            TAtomic::Scalar(TScalar::Integer(_)) => {
                let existing_integer = existing_atomic_type.get_integer();
                let assertion_integer = assertion_type.get_integer();

                match (existing_integer, assertion_integer) {
                    (Some(existing_integer), Some(assertion_integer)) => {
                        did_remove_type = true;

                        acceptable_types.extend(
                            existing_integer
                                .difference(assertion_integer, false)
                                .into_iter()
                                .map(|remaining_integer| TAtomic::Scalar(TScalar::Integer(remaining_integer))),
                        )
                    }
                    _ => {
                        acceptable_types.push(existing_atomic_type);
                    }
                }
            }
            TAtomic::Scalar(TScalar::Float(_)) => {
                let existing_value = existing_atomic_type.get_literal_float_value();
                let assertion_value = assertion_type.get_literal_float_value();

                match (existing_value, assertion_value) {
                    (Some(existing_value), Some(assertion_value)) => {
                        if existing_value == assertion_value {
                            did_remove_type = true;
                        } else {
                            acceptable_types.push(existing_atomic_type);
                        }
                    }
                    (None, Some(_)) => {
                        did_remove_type = true;
                        acceptable_types.push(existing_atomic_type);
                    }
                    _ => {
                        acceptable_types.push(existing_atomic_type);
                    }
                }
            }
            TAtomic::Scalar(TScalar::ArrayKey) => {
                if let TAtomic::Scalar(scalar) = assertion_type
                    && (scalar.is_known_literal_string() || scalar.is_literal_int())
                {
                    did_remove_type = true;
                }

                acceptable_types.push(existing_atomic_type);
            }
            TAtomic::Scalar(TScalar::ClassLikeString(_)) => {
                let existing_classlike_string = existing_atomic_type.get_class_string_value();
                let assertion_value = assertion_type.get_class_string_value();

                match (existing_classlike_string, assertion_value) {
                    (Some(existing_value), Some(assertion_value)) => {
                        if existing_value == assertion_value {
                            did_remove_type = true;
                        } else {
                            acceptable_types.push(existing_atomic_type);
                        }
                    }
                    (None, Some(_)) => {
                        did_remove_type = true;
                        acceptable_types.push(existing_atomic_type);
                    }
                    _ => {
                        acceptable_types.push(existing_atomic_type);
                    }
                }
            }
            _ => {
                acceptable_types.push(existing_atomic_type);
            }
        }
    }

    if let Some(key) = &key
        && let Some(pos) = span
        && (!did_remove_type || acceptable_types.is_empty())
    {
        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, pos);
    }

    if acceptable_types.is_empty() {
        return get_never();
    }

    new_var_type.types = Cow::Owned(acceptable_types);
    new_var_type
}
