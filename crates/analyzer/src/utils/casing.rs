use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Use;
use mago_syntax::ast::UseItems;
use mago_syntax::ast::UseType;

use crate::code::IssueCode;
use crate::context::Context;

/// Checks if the used class-like name matches the canonical casing and reports if not.
pub fn check_class_like_casing(context: &mut Context<'_, '_>, used_name: Atom, span: Span) {
    if !context.settings.check_name_casing {
        return;
    }

    let Some(metadata) = context.codebase.get_class_like(&used_name) else {
        return;
    };

    let canonical = metadata.original_name;
    if canonical == used_name {
        return;
    }

    // Extract the mismatched part (could be namespace or class name)
    // Compare only the parts that differ
    context.collector.report_with_code(
        IssueCode::IncorrectClassLikeCasing,
        Issue::warning(format!("Incorrect casing for class-like `{used_name}`."))
            .with_annotation(
                Annotation::primary(span).with_message(format!("Used as `{used_name}`, but defined as `{canonical}`.")),
            )
            .with_note("Using incorrect casing can cause autoloading failures on case-sensitive file systems.")
            .with_help(format!("Use the correct casing: `{canonical}`.")),
    );
}

/// Checks if the used function name matches the canonical casing and reports if not.
pub fn check_function_casing(context: &mut Context<'_, '_>, used_name: Atom, span: Span) {
    if !context.settings.check_name_casing {
        return;
    }

    let Some(metadata) = context.codebase.get_function(&used_name) else {
        return;
    };

    let Some(canonical) = metadata.original_name else {
        return;
    };

    if canonical == used_name {
        return;
    }

    context.collector.report_with_code(
        IssueCode::IncorrectFunctionCasing,
        Issue::warning(format!("Incorrect casing for function `{used_name}`."))
            .with_annotation(
                Annotation::primary(span).with_message(format!("Used as `{used_name}`, but defined as `{canonical}`.")),
            )
            .with_help(format!("Use the correct casing: `{canonical}`.")),
    );
}

/// Checks casing of all names imported via `use` statements.
pub fn check_use_statement_casing(context: &mut Context<'_, '_>, r#use: &Use<'_>) {
    match &r#use.items {
        UseItems::Sequence(sequence) => {
            for item in sequence.items.iter() {
                let fqn = atom(item.name.value().trim_start_matches('\\'));
                check_class_like_casing(context, fqn, item.name.span());
            }
        }
        UseItems::TypedSequence(typed_sequence) => {
            for item in typed_sequence.items.iter() {
                let fqn = atom(item.name.value().trim_start_matches('\\'));
                check_typed_use_casing(context, fqn, item.name.span(), &typed_sequence.r#type);
            }
        }
        UseItems::TypedList(typed_list) => {
            let prefix = typed_list.namespace.value().trim_start_matches('\\');
            for item in typed_list.items.iter() {
                let fqn = concat_atom!(prefix, "\\", item.name.value());
                check_typed_use_casing(context, fqn, item.name.span(), &typed_list.r#type);
            }
        }
        UseItems::MixedList(mixed_list) => {
            let prefix = mixed_list.namespace.value().trim_start_matches('\\');
            for maybe_typed_item in mixed_list.items.iter() {
                let fqn = concat_atom!(prefix, "\\", maybe_typed_item.item.name.value());
                match &maybe_typed_item.r#type {
                    Some(UseType::Function(_)) => {
                        check_function_casing(context, fqn, maybe_typed_item.item.name.span())
                    }
                    Some(UseType::Const(_)) => {} // Constants are case-sensitive
                    None => check_class_like_casing(context, fqn, maybe_typed_item.item.name.span()),
                }
            }
        }
    }
}

fn check_typed_use_casing(context: &mut Context<'_, '_>, fqn: Atom, span: Span, use_type: &UseType<'_>) {
    match use_type {
        UseType::Function(_) => check_function_casing(context, fqn, span),
        UseType::Const(_) => {} // Constants are case-sensitive
    }
}
