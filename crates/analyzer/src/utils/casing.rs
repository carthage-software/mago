use mago_allocator::Arena;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Use;
use mago_syntax::ast::UseItems;
use mago_syntax::ast::UseType;
use mago_word::Word;
use mago_word::concat_word;
use mago_word::word;

use crate::code::IssueCode;
use crate::context::Context;

/// Checks if the used class-like name matches the canonical casing and reports if not.
pub fn check_class_like_casing<A>(context: &mut Context<'_, '_, A>, used_name: Word, span: Span)
where
    A: Arena,
{
    if !context.settings.check_name_casing {
        return;
    }

    let Some(metadata) = context.codebase.get_class_like(used_name.as_bytes()) else {
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
pub fn check_function_casing_with_metadata<A>(
    context: &mut Context<'_, '_, A>,
    metadata: &FunctionLikeMetadata,
    used_name: Word,
    span: Span,
) where
    A: Arena,
{
    if !context.settings.check_name_casing {
        return;
    }

    let canonical = metadata.original_name;

    if canonical == used_name {
        return;
    }

    let canonical_bytes = canonical.as_bytes();
    let used_bytes = used_name.as_bytes();
    if memchr::memchr(b'\\', canonical_bytes).is_none()
        && let Some(idx) = memchr::memrchr(b'\\', used_bytes)
        && &used_bytes[idx + 1..] == canonical_bytes
    {
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

/// Checks if the used function name matches the canonical casing and reports if not.
fn check_function_casing<A>(context: &mut Context<'_, '_, A>, used_name: Word, span: Span)
where
    A: Arena,
{
    if !context.settings.check_name_casing {
        return;
    }

    let Some(metadata) = context.codebase.get_function(used_name.as_bytes()) else {
        return;
    };

    check_function_casing_with_metadata(context, metadata, used_name, span);
}

/// Checks casing of all names imported via `use` statements.
pub fn check_use_statement_casing<A>(context: &mut Context<'_, '_, A>, r#use: &Use<'_>)
where
    A: Arena,
{
    match &r#use.items {
        UseItems::Sequence(sequence) => {
            for item in sequence.items.iter() {
                let fqn = word(mago_bytes::trim_start_byte(item.name.value(), b'\\'));
                check_class_like_casing(context, fqn, item.name.span());
            }
        }
        UseItems::TypedSequence(typed_sequence) => {
            for item in typed_sequence.items.iter() {
                let fqn = word(mago_bytes::trim_start_byte(item.name.value(), b'\\'));
                check_typed_use_casing(context, fqn, item.name.span(), &typed_sequence.r#type);
            }
        }
        UseItems::TypedList(typed_list) => {
            let prefix = mago_bytes::trim_start_byte(typed_list.namespace.value(), b'\\');
            for item in typed_list.items.iter() {
                let fqn = concat_word!(prefix, b"\\", item.name.value());
                check_typed_use_casing(context, fqn, item.name.span(), &typed_list.r#type);
            }
        }
        UseItems::MixedList(mixed_list) => {
            let prefix = mago_bytes::trim_start_byte(mixed_list.namespace.value(), b'\\');
            for maybe_typed_item in mixed_list.items.iter() {
                let fqn = concat_word!(prefix, b"\\", maybe_typed_item.item.name.value());
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

fn check_typed_use_casing<A>(context: &mut Context<'_, '_, A>, fqn: Word, span: Span, use_type: &UseType<'_>)
where
    A: Arena,
{
    match use_type {
        UseType::Function(_) => check_function_casing(context, fqn, span),
        UseType::Const(_) => {} // Constants are case-sensitive
    }
}
