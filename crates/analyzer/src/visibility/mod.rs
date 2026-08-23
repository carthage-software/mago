use mago_allocator::Arena;
use mago_word::Word;
use mago_word::word;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::property::PropertyMetadata;
use mago_codex::visibility::Visibility;
use mago_php_version::PHPVersion;
use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::resolver::property::DeclaredProperty;
use crate::resolver::property::DeclaredPropertyKind;
use crate::resolver::property::resolve_declared_property;
use mago_bytes::BytesDisplay;

/// Checks if a method is visible from the current scope and reports a detailed
/// error if it is not.
///
/// # Arguments
///
/// * `context` - The global analysis context.
/// * `calling_class` - The class the access occurs in, or `None` for the global scope.
/// * `fqcn` - The fully-qualified class name on which the method is being called.
/// * `method_name` - The method name.
/// * `access_span` - The span of the entire method call/access expression (e.g., `$obj->method()`).
/// * `method_name_span` - The span of just the method name identifier (e.g., `method`).
///
/// # Returns
///
/// `true` if the method is visible, `false` otherwise. An error is reported to the
/// context buffer if the method is not visible.
pub fn check_method_visibility<A>(
    context: &mut Context<'_, '_, A>,
    calling_class: Option<Word>,
    fqcn: &[u8],
    method_name: &[u8],
    access_span: Span,
    member_span: Option<Span>,
) -> bool
where
    A: Arena,
{
    let declaring_class = context.codebase.get_declaring_method_class(fqcn, method_name).unwrap_or_else(|| word(fqcn));

    let Some(method_metadata) = context.codebase.get_declaring_method(fqcn, method_name) else {
        return true;
    };

    // Get the effective visibility, checking trait alias visibility overrides
    let Some(visibility) = context.codebase.get_method_visibility(fqcn, method_name) else {
        return true;
    };

    if visibility == Visibility::Public {
        return true;
    }

    let is_visible = is_visible_from_scope(context.codebase, visibility, declaring_class.as_bytes(), calling_class);
    if !is_visible {
        let declaring_class_name = context
            .codebase
            .get_class_like(declaring_class.as_bytes())
            .map_or_else(|| declaring_class, |metadata| metadata.original_name);

        let issue_title =
            format!("Cannot access {} method `{}::{}`.", visibility, declaring_class_name, BytesDisplay(method_name));
        let help_text = format!(
            "Change the visibility of method `{}` to `public`, or call it from an allowed scope.",
            BytesDisplay(method_name)
        );

        report_visibility_issue(
            context,
            calling_class,
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

/// Determines whether a method is visible from the current scope
///
/// Returns `true` when the method is visible (or when visibility cannot be determined), and
/// `false` only when the method definitively exists but is inaccessible from the current scope.
pub fn is_method_visible<'ctx, A>(
    context: &Context<'ctx, '_, A>,
    block_context: &BlockContext<'ctx>,
    fqcn: &[u8],
    method_name: &[u8],
) -> bool
where
    A: Arena,
{
    let declaring_class = context.codebase.get_declaring_method_class(fqcn, method_name).unwrap_or_else(|| word(fqcn));

    if context.codebase.get_declaring_method(fqcn, method_name).is_none() {
        return true;
    }

    let Some(visibility) = context.codebase.get_method_visibility(fqcn, method_name) else {
        return true;
    };

    if visibility == Visibility::Public {
        return true;
    }

    is_visible_from_scope(
        context.codebase,
        visibility,
        declaring_class.as_bytes(),
        block_context.scope.get_class_like_name(),
    )
}

/// Checks if a static property (`Class::$prop`) is readable from the current scope and
/// reports a detailed error if it is not.  Magic `@property*` annotations never apply to
/// static access, so this entry point resolves annotation-blind; instance accesses go through
/// [`check_resolved_property_read_visibility`] with a resolution from
/// [`resolve_declared_property`] instead.
pub fn check_static_property_read_visibility<'ctx, A>(
    context: &mut Context<'ctx, '_, A>,
    block_context: &BlockContext<'ctx>,
    fqcn: &[u8],
    property_name: &[u8],
    access_span: Span,
    member_span: Option<Span>,
) -> bool
where
    A: Arena,
{
    let property_name = word(property_name);

    let Some(class_metadata) = context.codebase.get_class_like(fqcn) else {
        return true;
    };

    let Some(resolution) = resolve_declared_property(
        context.codebase,
        class_metadata,
        property_name,
        false, // `instance_access`: this entry point serves `Class::$prop` resolution
        block_context.scope.get_class_like_name(),
    ) else {
        return true;
    };

    check_resolved_property_read_visibility(context, block_context, &resolution, access_span, member_span)
}

/// Checks if a property access already resolved by [`resolve_declared_property`] is readable
/// from the current scope and reports a detailed error if it is not.
pub(crate) fn check_resolved_property_read_visibility<'ctx, A>(
    context: &mut Context<'ctx, '_, A>,
    block_context: &BlockContext<'ctx>,
    resolution: &DeclaredProperty<'_>,
    access_span: Span,
    member_span: Option<Span>,
) -> bool
where
    A: Arena,
{
    let DeclaredProperty { declaring_class: declaring_class_metadata, property: property_metadata, kind } = *resolution;
    let property_name = property_metadata.name.0;

    if matches!(kind, DeclaredPropertyKind::Magic) {
        if !property_metadata.flags.is_writeonly() {
            return true;
        }

        let class_name = &declaring_class_metadata.original_name;

        context.collector.report_with_code(
            IssueCode::InvalidPropertyRead,
            Issue::error(format!(
                "Cannot read from write-only property `{class_name}::{property_name}`."
            ))
            .with_annotation(
                Annotation::primary(member_span.unwrap_or(access_span))
                    .with_message("Attempt to read from a write-only property"),
            )
            .with_annotation(
                Annotation::secondary(declaring_class_metadata.name_span.unwrap_or(declaring_class_metadata.span))
                    .with_message(format!("Property is defined as write-only via a `@property-write` tag on class `{class_name}`")),
            )
            .with_note("Properties defined with `@property-write` are 'magic' properties that can be assigned to, but not read from.")
            .with_help("If this property should be readable, change its docblock definition from `@property-write` to `@property`."),
        );

        return false;
    }

    if !property_metadata.hooks.is_empty()
        && property_metadata.hooks.contains_key(&word(b"set"))
        && !property_metadata.hooks.contains_key(&word(b"get"))
        && property_metadata.flags.is_virtual_property()
    {
        let class_name = &declaring_class_metadata.original_name;

        context.collector.report_with_code(
            IssueCode::InvalidPropertyRead,
            Issue::error(format!(
                "Cannot read from write-only property `{class_name}::{property_name}` - property only has a set hook."
            ))
            .with_annotation(Annotation::primary(member_span.unwrap_or(access_span)).with_message("Read access here"))
            .with_annotation(
                Annotation::secondary(property_metadata.span.or(property_metadata.name_span).unwrap_or(access_span))
                    .with_message("Property defined here with only a set hook"),
            )
            .with_help("Add a get hook to make this property readable."),
        );

        return false;
    }

    let visibility = property_metadata.read_visibility;
    let is_visible = is_visible_from_scope(
        context.codebase,
        visibility,
        declaring_class_metadata.name.as_bytes(),
        block_context.scope.get_class_like_name(),
    );

    if !is_visible {
        let issue_title = format!(
            "Cannot read {} property `{}` from class `{}`.",
            visibility, property_name, declaring_class_metadata.original_name
        );

        let help_text =
            format!("Make the property `{property_name}` readable (e.g., `public`), or add a public getter method.");

        report_visibility_issue(
            context,
            block_context.scope.get_class_like_name(),
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

/// The write-access counterpart of [`check_resolved_property_read_visibility`].
pub(crate) fn check_resolved_property_write_visibility<'ctx, A>(
    context: &mut Context<'ctx, '_, A>,
    block_context: &BlockContext<'ctx>,
    resolution: &DeclaredProperty<'_>,
    access_span: Span,
    member_span: Option<Span>,
) -> bool
where
    A: Arena,
{
    let DeclaredProperty { declaring_class: declaring_class_metadata, property: property_metadata, kind } = *resolution;
    let property_name = property_metadata.name.0;

    if matches!(kind, DeclaredPropertyKind::Magic) {
        if property_metadata.flags.is_readonly() {
            let class_name = &declaring_class_metadata.original_name;
            context.collector.report_with_code(
                IssueCode::InvalidPropertyWrite,
                Issue::error(format!("Cannot write to documented read-only property `{class_name}::{property_name}`."))
                    .with_annotation(
                        Annotation::primary(member_span.unwrap_or(access_span)).with_message("This property is read-only"),
                    )
                    .with_help("Remove the assignment or change the property documentation to `@property` if writes are supported."),
            );
        }

        // Keep resolving the access so the resolver can independently report a
        // missing `__set()` implementation when applicable.
        return true;
    }

    if !property_metadata.hooks.is_empty()
        && property_metadata.hooks.contains_key(&word(b"get"))
        && !property_metadata.hooks.contains_key(&word(b"set"))
        && property_metadata.flags.is_virtual_property()
    {
        let class_name = &declaring_class_metadata.original_name;

        context.collector.report_with_code(
            IssueCode::InvalidPropertyWrite,
            Issue::error(format!(
                "Cannot write to read-only property `{class_name}::{property_name}` - property only has a get hook."
            ))
            .with_annotation(Annotation::primary(member_span.unwrap_or(access_span)).with_message("Write access here"))
            .with_annotation(
                Annotation::secondary(property_metadata.span.or(property_metadata.name_span).unwrap_or(access_span))
                    .with_message("Property defined here with only a get hook"),
            )
            .with_help("Add a set hook to make this property writable."),
        );

        return false;
    }

    let visibility = effective_write_visibility(property_metadata, context.settings.version);

    let is_visible = is_visible_from_scope(
        context.codebase,
        visibility,
        declaring_class_metadata.name.as_bytes(),
        block_context.scope.get_class_like_name(),
    );
    if !is_visible {
        if property_metadata.flags.is_readonly() {
            report_readonly_write_scope_issue(
                context,
                block_context.scope.get_class_like_name(),
                declaring_class_metadata,
                property_metadata,
                visibility,
                access_span,
                member_span,
            );
        } else {
            let issue_title = format!(
                "Cannot write to {} property `{}` on class `{}`.",
                visibility, property_name, declaring_class_metadata.original_name
            );

            let help_text = format!(
                "Make the property `{property_name}` writable (e.g., `public` or `public(set)`), or add a public setter method."
            );

            report_visibility_issue(
                context,
                block_context.scope.get_class_like_name(),
                IssueCode::InvalidPropertyWrite,
                issue_title,
                visibility,
                access_span,
                member_span,
                property_metadata.span.or(property_metadata.name_span),
                help_text,
            );
        }
    }

    is_visible
}

/// Reports a write to a `readonly` property from a scope that cannot initialize it.
///
/// `readonly`, not the declared visibility, is what forbids the write here, so naming the
/// effective write visibility would name one the declaration does not carry.
#[allow(clippy::too_many_arguments)]
fn report_readonly_write_scope_issue<A>(
    context: &mut Context<'_, '_, A>,
    calling_class: Option<Word>,
    declaring_class: &ClassLikeMetadata,
    property: &PropertyMetadata,
    visibility: Visibility,
    access_span: Span,
    member_span: Option<Span>,
) where
    A: Arena,
{
    let class_name = &declaring_class.original_name;
    let property_name = property.name.0;

    let allowed_scope = if visibility == Visibility::Private {
        format!("`{class_name}` itself")
    } else {
        format!("`{class_name}` or a class in its hierarchy")
    };

    let current_scope = match calling_class {
        Some(current_class) => {
            let current_class_name = context
                .codebase
                .get_class_like(current_class.as_bytes())
                .map_or(current_class, |metadata| metadata.original_name);

            format!("from within `{current_class_name}`")
        }
        None => "from the global scope".to_string(),
    };

    let mut issue =
        Issue::error(format!("Cannot initialize readonly property `{class_name}::{property_name}` {current_scope}."))
            .with_annotation(
                Annotation::primary(member_span.unwrap_or(access_span))
                    .with_message(format!("Only {allowed_scope} may initialize this readonly property")),
            )
            .with_annotation(
                Annotation::secondary(access_span).with_message(format!("Invalid write occurs here, {current_scope}")),
            );

    if let Some(definition_span) = property.span.or(property.name_span) {
        issue =
            issue.with_annotation(Annotation::secondary(definition_span).with_message("Property is `readonly` here"));
    }

    let (note, help) = if visibility == Visibility::Private {
        (
            format!(
                "Before PHP 8.4, a readonly property can only be initialized from the scope of `{class_name}`; every other write throws an `Error` at runtime."
            ),
            format!(
                "Initialize the property in `{class_name}`, for example in its constructor, or add a method there that performs the write."
            ),
        )
    } else {
        (
            "A readonly property that does not declare `public(set)` is limited to `protected(set)` writes; an assignment from an unrelated scope throws an `Error` at runtime.".to_string(),
            format!(
                "Initialize the property in `{class_name}` or a class derived from it, or declare it `public(set) readonly` to allow initialization from any scope."
            ),
        )
    };

    context.collector.report_with_code(IssueCode::InvalidPropertyWrite, issue.with_note(note).with_help(help));
}

/// The visibility that governs writes to `property`, which is narrower than the
/// declared write visibility for `readonly` properties.
pub(crate) fn effective_write_visibility(property: &PropertyMetadata, version: PHPVersion) -> Visibility {
    if !property.flags.is_readonly() {
        return property.write_visibility;
    }

    if !version.is_supported(Feature::AsymmetricVisibility) {
        return Visibility::Private;
    }

    match property.write_visibility {
        Visibility::Public => Visibility::Protected,
        visibility => visibility,
    }
}

pub(crate) fn is_visible_from_scope(
    codebase: &CodebaseMetadata,
    visibility: Visibility,
    declaring_class_id: &[u8],
    current_class_opt: Option<Word>,
) -> bool {
    match visibility {
        Visibility::Public => true,
        Visibility::Protected => {
            if let Some(current_class_id) = current_class_opt {
                current_class_id.as_bytes().eq_ignore_ascii_case(declaring_class_id)
                    || codebase.is_instance_of(current_class_id.as_bytes(), declaring_class_id)
                    || codebase.is_instance_of(declaring_class_id, current_class_id.as_bytes())
                    || is_visible_via_required_extends(codebase, current_class_id.as_bytes(), declaring_class_id)
            } else {
                false
            }
        }
        Visibility::Private => {
            if let Some(current_class_id) = current_class_opt {
                current_class_id.as_bytes().eq_ignore_ascii_case(declaring_class_id)
                    || codebase.class_uses_trait(current_class_id.as_bytes(), declaring_class_id)
                    || codebase.class_uses_trait(declaring_class_id, current_class_id.as_bytes())
            } else {
                false
            }
        }
    }
}

/// Checks if a protected member declared in `declaring_class_id` is accessible from
/// `current_class_id` via `@require-extends`. This handles the case where a trait has
/// `@require-extends BaseClass` and `BaseClass` uses another trait that declares the method.
fn is_visible_via_required_extends(
    codebase: &CodebaseMetadata,
    current_class_id: &[u8],
    declaring_class_id: &[u8],
) -> bool {
    let current_class_id_lc = mago_word::ascii_lowercase_word(current_class_id);

    let Some(current_metadata) = codebase.get_class_like(current_class_id_lc.as_bytes()) else {
        return false;
    };

    if current_metadata.require_extends.is_empty() {
        return false;
    }

    for required_class in current_metadata.require_extends.iter() {
        if codebase.is_instance_of(required_class.as_bytes(), declaring_class_id)
            || codebase.class_uses_trait(required_class.as_bytes(), declaring_class_id)
        {
            return true;
        }
    }

    false
}

fn report_visibility_issue<A>(
    context: &mut Context<'_, '_, A>,
    calling_class: Option<Word>,
    code: IssueCode,
    title: String,
    visibility: Visibility,
    access_span: Span,
    member_span: Option<Span>,
    definition_span: Option<Span>,
    help_text: String,
) where
    A: Arena,
{
    let current_scope_str = if let Some(current_class) = calling_class {
        format!("from within `{current_class}`")
    } else {
        "from the global scope".to_string()
    };

    let primary_annotation_span = member_span.unwrap_or(access_span);

    let mut issue = Issue::error(title)
        .with_annotation(
            Annotation::primary(primary_annotation_span)
                .with_message(format!("This member is {} and cannot be accessed here", visibility)),
        )
        .with_annotation(
            Annotation::secondary(access_span).with_message(format!("Invalid access occurs here, {current_scope_str}")),
        );

    if let Some(definition_span) = definition_span
        && definition_span != primary_annotation_span
    {
        issue = issue.with_annotation(
            Annotation::secondary(definition_span).with_message(format!("Member is defined as `{}` here", visibility)),
        );
    }

    issue = issue.with_help(help_text);

    context.collector.report_with_code(code, issue);
}
