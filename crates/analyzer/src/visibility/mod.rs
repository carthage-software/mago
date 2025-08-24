use mago_atom::Atom;
use mago_atom::atom;
use mago_codex::get_class_like;
use mago_codex::get_declaring_method_identifier;
use mago_codex::get_method_by_id;
use mago_codex::get_method_identifier;
use mago_codex::inherits_class;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::method_identifier_exists;
use mago_codex::uses_trait;
use mago_codex::visibility::Visibility;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;

/// Checks if a method is visible from the current scope and reports a detailed
/// error if it is not.
///
/// # Arguments
///
/// * `context` - The global analysis context.
/// * `block_context` - The context of the current code block, providing scope information.
/// * `fqcn` - The fully-qualified class name on which the method is being called.
/// * `method_name` - The method name.
/// * `access_span` - The span of the entire method call/access expression (e.g., `$obj->method()`).
/// * `method_name_span` - The span of just the method name identifier (e.g., `method`).
///
/// # Returns
///
/// `true` if the method is visible, `false` otherwise. An error is reported to the
/// context buffer if the method is not visible.
pub fn check_method_visibility<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    fqcn: &str,
    method_name: &str,
    access_span: Span,
    member_span: Option<Span>,
) -> bool {
    let mut method_id = get_method_identifier(fqcn, method_name);
    if !method_identifier_exists(context.codebase, &method_id) {
        method_id = get_declaring_method_identifier(context.codebase, &method_id);
    }

    let Some(method_metadata) = get_method_by_id(context.codebase, &method_id) else {
        return true;
    };

    let Some(visibility) = method_metadata.method_metadata.as_ref().map(|m| m.visibility) else {
        return true;
    };

    if visibility == Visibility::Public {
        return true;
    }

    let declaring_class_id = method_id.get_class_name();

    let is_visible =
        is_visible_from_scope(context, visibility, declaring_class_id, block_context.scope.get_class_like_name());

    if !is_visible {
        let declaring_class_name = get_class_like(context.codebase, declaring_class_id)
            .map(|metadata| metadata.original_name)
            .unwrap_or_else(|| *declaring_class_id);

        let issue_title =
            format!("Cannot access {} method `{}::{}`.", visibility.as_str(), declaring_class_name, method_name);
        let help_text =
            format!("Change the visibility of method `{method_name}` to `public`, or call it from an allowed scope.");

        report_visibility_issue(
            context,
            block_context,
            IssueCode::InvalidMethodAccess,
            issue_title,
            visibility,
            access_span,
            member_span,
            Some(method_metadata.span),
            help_text,
        );
    }

    is_visible
}

/// Checks if a property is readable from the current scope and reports a detailed
/// error if it is not.
pub fn check_property_read_visibility<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    fqcn: &str,
    property_name: &str,
    access_span: Span,
    member_span: Option<Span>,
) -> bool {
    let property_name = atom(property_name);

    let Some(class_metadata) = get_class_like(context.codebase, fqcn) else {
        return true;
    };

    let Some(declaring_class_id) = class_metadata.declaring_property_ids.get(&property_name) else {
        return true;
    };

    let Some(declaring_class_metadata) = get_class_like(context.codebase, declaring_class_id) else {
        return true;
    };

    let Some(property_metadata) = declaring_class_metadata.properties.get(&property_name) else {
        return true;
    };

    let visibility = property_metadata.read_visibility;
    let is_visible =
        is_visible_from_scope(context, visibility, declaring_class_id, block_context.scope.get_class_like_name());

    if !is_visible {
        let issue_title = format!(
            "Cannot read {} property `{}` from class `{}`.",
            visibility.as_str(),
            property_name,
            declaring_class_metadata.original_name
        );

        let help_text =
            format!("Make the property `{property_name}` readable (e.g., `public`), or add a public getter method.");

        report_visibility_issue(
            context,
            block_context,
            IssueCode::InvalidPropertyRead,
            issue_title,
            visibility,
            access_span,
            member_span,
            property_metadata.span.or(property_metadata.name_span),
            help_text,
        );
    }

    is_visible
}

pub fn check_property_write_visibility<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    fqcn: &str,
    property_name: &str,
    access_span: Span,
    member_span: Option<Span>,
) -> bool {
    let property_name = atom(property_name);

    let Some(class_metadata) = get_class_like(context.codebase, fqcn) else {
        return true;
    };

    let Some(declaring_class_name) = class_metadata.declaring_property_ids.get(&property_name) else {
        return true;
    };

    let Some(declaring_class_metadata) = get_class_like(context.codebase, declaring_class_name) else {
        return true;
    };

    let Some(property_metadata) = declaring_class_metadata.properties.get(&property_name) else {
        return true;
    };

    let visibility = property_metadata.write_visibility;
    let is_visible =
        is_visible_from_scope(context, visibility, declaring_class_name, block_context.scope.get_class_like_name());

    if !is_visible {
        let issue_title = format!(
            "Cannot write to {} property `{}` on class `{}`.",
            visibility.as_str(),
            property_name,
            declaring_class_metadata.original_name
        );

        let help_text = format!(
            "Make the property `{property_name}` writable (e.g., `public` or `public(set)`), or add a public setter method."
        );

        report_visibility_issue(
            context,
            block_context,
            IssueCode::InvalidPropertyWrite,
            issue_title,
            visibility,
            access_span,
            member_span,
            property_metadata.span.or(property_metadata.name_span),
            help_text,
        );
    } else if property_metadata.flags.is_readonly()
        && !can_initialize_readonly_property(
            context,
            declaring_class_name,
            block_context.scope.get_class_like_name(),
            block_context.scope.get_function_like(),
        )
    {
        report_readonly_issue(
            context,
            block_context,
            IssueCode::InvalidPropertyWrite,
            access_span,
            member_span,
            property_metadata.span.or(property_metadata.name_span),
        );
    }

    is_visible
}

fn is_visible_from_scope(
    context: &Context<'_, '_>,
    visibility: Visibility,
    declaring_class_id: &str,
    current_class_opt: Option<Atom>,
) -> bool {
    match visibility {
        Visibility::Public => true,
        Visibility::Protected => {
            if let Some(current_class_id) = current_class_opt {
                current_class_id.eq_ignore_ascii_case(declaring_class_id)
                    || inherits_class(context.codebase, &current_class_id, declaring_class_id)
                    || inherits_class(context.codebase, declaring_class_id, &current_class_id)
                    || uses_trait(context.codebase, &current_class_id, declaring_class_id)
                    || uses_trait(context.codebase, declaring_class_id, &current_class_id)
            } else {
                false
            }
        }
        Visibility::Private => {
            if let Some(current_class_id) = current_class_opt {
                current_class_id.eq_ignore_ascii_case(declaring_class_id)
                    || uses_trait(context.codebase, &current_class_id, declaring_class_id)
                    || uses_trait(context.codebase, declaring_class_id, &current_class_id)
            } else {
                false
            }
        }
    }
}

fn can_initialize_readonly_property(
    context: &Context<'_, '_>,
    declaring_class_id: &str,
    current_class_opt: Option<Atom>,
    current_function_opt: Option<&FunctionLikeMetadata>,
) -> bool {
    current_function_opt.and_then(|func| func.method_metadata.as_ref()).is_some_and(|method| method.is_constructor)
        && current_class_opt.is_some_and(|current_class_id| {
            current_class_id.eq_ignore_ascii_case(declaring_class_id)
                || inherits_class(context.codebase, &current_class_id, declaring_class_id)
                || inherits_class(context.codebase, declaring_class_id, &current_class_id)
                || uses_trait(context.codebase, &current_class_id, declaring_class_id)
                || uses_trait(context.codebase, declaring_class_id, &current_class_id)
        })
}

fn report_visibility_issue<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    code: IssueCode,
    title: String,
    visibility: Visibility,
    access_span: Span,
    member_span: Option<Span>,
    definition_span: Option<Span>,
    help_text: String,
) {
    let current_scope_str = if let Some(current_class) = block_context.scope.get_class_like_name() {
        format!("from within `{current_class}`")
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

    if let Some(definition_span) = definition_span
        && definition_span != primary_annotation_span
    {
        issue = issue.with_annotation(
            Annotation::secondary(definition_span)
                .with_message(format!("Member is defined as `{}` here", visibility.as_str())),
        );
    }

    issue = issue.with_help(help_text);

    context.collector.report_with_code(code, issue);
}

fn report_readonly_issue<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    code: IssueCode,
    access_span: Span,
    member_span: Option<Span>,
    definition_span: Option<Span>,
) {
    let current_scope_str = if let Some(current_class) = block_context.scope.get_class_like_name() {
        format!("from within `{current_class}`")
    } else {
        "from the global scope".to_string()
    };

    let primary_annotation_span = member_span.unwrap_or(access_span);

    let mut issue = Issue::error("Cannot modify a readonly property after initialization.")
        .with_annotation(
            Annotation::primary(primary_annotation_span)
                .with_message("Illegal write to readonly property"),
        )
        .with_annotation(
            Annotation::secondary(access_span).with_message(format!("Write attempt occurs here, {current_scope_str}")),
        )
        .with_note(
            "Readonly properties can only be initialized once, and this must occur within the scope of the class that declares them (i.e., in its `__construct` method or a child's `__construct`)."
        )
        .with_help(
            "While PHP may permit this write if the property is uninitialized, it will cause a fatal `Error` if the property already has a value. To ensure correctness, move this initialization to the constructor."
        );

    if let Some(definition_span) = definition_span {
        issue = issue.with_annotation(
            Annotation::secondary(definition_span).with_message("Property is defined as `readonly` here"),
        );
    }

    context.collector.report_with_code(code, issue);
}
