use mago_codex::get_class_like;
use mago_codex::get_declaring_method_id;
use mago_codex::get_method_by_id;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::inherits_class;
use mago_codex::method_id_exists;
use mago_codex::uses_trait;
use mago_codex::visibility::Visibility;
use mago_interner::StringIdentifier;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::context::Context;
use crate::context::block::BlockContext;
use crate::issue::TypingIssueKind;

/// Checks if a method is visible from the current scope and reports a detailed
/// error if it is not.
///
/// # Arguments
///
/// * `context` - The global analysis context.
/// * `block_context` - The context of the current code block, providing scope information.
/// * `fqc_id` - The fully-qualified class name on which the method is being called.
/// * `method_name_id` - The identifier for the method name.
/// * `access_span` - The span of the entire method call/access expression (e.g., `$obj->method()`).
/// * `method_name_span` - The span of just the method name identifier (e.g., `method`).
///
/// # Returns
///
/// `true` if the method is visible, `false` otherwise. An error is reported to the
/// context buffer if the method is not visible.
pub fn check_method_visibility<'a>(
    context: &mut Context<'a>,
    block_context: &BlockContext<'a>,
    fqc_id: &StringIdentifier,
    method_name_id: &StringIdentifier,
    access_span: Span,
    member_span: Option<Span>,
) -> bool {
    let mut method_id = MethodIdentifier::new(*fqc_id, *method_name_id);
    if !method_id_exists(context.codebase, context.interner, &method_id) {
        method_id = get_declaring_method_id(
            context.codebase,
            context.interner,
            &MethodIdentifier::new(*fqc_id, *method_name_id),
        );
    }

    let Some(method_metadata) = get_method_by_id(context.codebase, context.interner, &method_id) else {
        return true;
    };

    let Some(visibility) = method_metadata.get_visibility() else {
        return true;
    };

    if visibility == Visibility::Public {
        return true;
    }

    let declaring_class_id = method_id.get_class_name();

    let is_visible =
        is_visible_from_scope(context, visibility, declaring_class_id, block_context.scope.get_class_like_name());

    if !is_visible {
        let method_name_str = context.interner.lookup(method_name_id);
        let declaring_class_name_str = context.interner.lookup(declaring_class_id);
        let issue_title = format!(
            "Cannot access {} method `{}::{}`.",
            visibility.as_str(),
            declaring_class_name_str,
            method_name_str
        );
        let help_text = format!(
            "Change the visibility of method `{method_name_str}` to `public`, or call it from an allowed scope."
        );

        report_visibility_issue(
            context,
            block_context,
            TypingIssueKind::InvalidMethodAccess,
            issue_title,
            visibility,
            access_span,
            member_span,
            method_metadata.get_span(),
            help_text,
        );
    }

    is_visible
}

/// Checks if a property is readable from the current scope and reports a detailed
/// error if it is not.
pub fn check_property_read_visibility<'a>(
    context: &mut Context<'a>,
    block_context: &BlockContext<'a>,
    fqc_id: &StringIdentifier,
    property_id: &StringIdentifier,
    access_span: Span,
    member_span: Option<Span>,
) -> bool {
    let Some(class_metadata) = get_class_like(context.codebase, context.interner, fqc_id) else {
        return true;
    };

    let Some(declaring_class_id) = class_metadata.declaring_property_ids.get(property_id) else {
        return true;
    };

    let Some(declaring_class_metadata) = get_class_like(context.codebase, context.interner, declaring_class_id) else {
        return true;
    };

    let Some(property_metadata) = declaring_class_metadata.properties.get(property_id) else {
        return true;
    };

    let visibility = property_metadata.read_visibility;
    let is_visible =
        is_visible_from_scope(context, visibility, declaring_class_id, block_context.scope.get_class_like_name());

    if !is_visible {
        let property_name_str = context.interner.lookup(property_id);
        let declaring_class_name_str = context.interner.lookup(declaring_class_id);

        let issue_title = format!(
            "Cannot read {} property `{}` from class `{}`.",
            visibility.as_str(),
            property_name_str,
            declaring_class_name_str
        );

        let help_text = format!(
            "Make the property `{property_name_str}` readable (e.g., `public`), or add a public getter method."
        );

        report_visibility_issue(
            context,
            block_context,
            TypingIssueKind::InvalidPropertyRead,
            issue_title,
            visibility,
            access_span,
            member_span,
            property_metadata.span.unwrap_or_else(|| property_metadata.name_span.unwrap()),
            help_text,
        );
    }

    is_visible
}

pub fn check_property_write_visibility<'a>(
    context: &mut Context<'a>,
    block_context: &BlockContext<'a>,
    fqc_id: &StringIdentifier,
    property_id: &StringIdentifier,
    access_span: Span,
    member_span: Option<Span>,
) -> bool {
    let Some(class_metadata) = get_class_like(context.codebase, context.interner, fqc_id) else {
        return true;
    };

    let Some(declaring_class_id) = class_metadata.declaring_property_ids.get(property_id) else {
        return true;
    };

    let Some(declaring_class_metadata) = get_class_like(context.codebase, context.interner, declaring_class_id) else {
        return true;
    };

    let Some(property_metadata) = declaring_class_metadata.properties.get(property_id) else {
        return true;
    };

    let visibility = property_metadata.write_visibility;
    let is_visible =
        is_visible_from_scope(context, visibility, declaring_class_id, block_context.scope.get_class_like_name());

    if !is_visible {
        let property_name_str = context.interner.lookup(property_id);
        let declaring_class_name_str = context.interner.lookup(declaring_class_id);
        let issue_title = format!(
            "Cannot write to {} property `{}` on class `{}`.",
            visibility.as_str(),
            property_name_str,
            declaring_class_name_str
        );

        let help_text = format!(
            "Make the property `{property_name_str}` writable (e.g., `public` or `public(set)`), or add a public setter method."
        );

        report_visibility_issue(
            context,
            block_context,
            TypingIssueKind::InvalidPropertyWrite,
            issue_title,
            visibility,
            access_span,
            member_span,
            property_metadata.span.unwrap_or_else(|| property_metadata.name_span.unwrap()),
            help_text,
        );
    }

    is_visible
}

fn is_visible_from_scope(
    context: &Context<'_>,
    visibility: Visibility,
    declaring_class_id: &StringIdentifier,
    current_class_id_opt: Option<&StringIdentifier>,
) -> bool {
    match visibility {
        Visibility::Public => true,
        Visibility::Protected => {
            if let Some(current_class_id) = current_class_id_opt {
                context.interner.lowered(current_class_id) == context.interner.lowered(declaring_class_id)
                    || inherits_class(context.codebase, context.interner, current_class_id, declaring_class_id)
                    || inherits_class(context.codebase, context.interner, declaring_class_id, current_class_id)
                    || uses_trait(context.codebase, context.interner, current_class_id, declaring_class_id)
                    || uses_trait(context.codebase, context.interner, declaring_class_id, current_class_id)
            } else {
                false
            }
        }
        Visibility::Private => {
            if let Some(current_class_id) = current_class_id_opt {
                context.interner.lowered(current_class_id) == context.interner.lowered(declaring_class_id)
                    || uses_trait(context.codebase, context.interner, current_class_id, declaring_class_id)
                    || uses_trait(context.codebase, context.interner, declaring_class_id, current_class_id)
            } else {
                false
            }
        }
    }
}

fn report_visibility_issue(
    context: &mut Context<'_>,
    block_context: &BlockContext<'_>,
    kind: TypingIssueKind,
    title: String,
    visibility: Visibility,
    access_span: Span,
    member_span: Option<Span>,
    definition_span: Span,
    help_text: String,
) {
    let current_scope_str = if let Some(current_class_id) = block_context.scope.get_class_like_name() {
        format!("from within `{}`", context.interner.lookup(current_class_id))
    } else {
        "from the global scope".to_string()
    };

    let primary_annotation_span = member_span.unwrap_or(access_span);

    let mut issue = Issue::error(title)
        .with_annotation(
            Annotation::primary(primary_annotation_span)
                .with_message(format!("This member is {} and cannot be accessed here", visibility.as_str())),
        )
        .with_annotation(
            Annotation::secondary(access_span).with_message(format!("Invalid access occurs here, {current_scope_str}")),
        );

    if definition_span != primary_annotation_span {
        issue = issue.with_annotation(
            Annotation::secondary(definition_span)
                .with_message(format!("Member is defined as `{}` here", visibility.as_str())),
        );
    }

    issue = issue.with_help(help_text);

    context.buffer.report(kind, issue);
}
