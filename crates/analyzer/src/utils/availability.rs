use mago_allocator::Arena;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::class_like_constant::ClassLikeConstantMetadata;
use mago_codex::metadata::constant::ConstantMetadata;
use mago_codex::metadata::enum_case::EnumCaseMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::property::PropertyMetadata;
use mago_codex::metadata::version_constraint::VersionConstraint;
use mago_codex::symbol::SymbolKind;
use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::code::IssueCode;
use crate::context::Context;

pub fn check_availability<A>(
    context: &mut Context<'_, '_, A>,
    code: IssueCode,
    kind: &str,
    version_constraint: &VersionConstraint,
    display_name: &(impl std::fmt::Display + ?Sized),
    span: Span,
) where
    A: Arena,
{
    let version = context.settings.version;
    if version_constraint.allows_version(version) {
        return;
    }

    report_unavailable(context, code, kind, display_name, version_constraint, version, span);
}

/// Reports a "symbol unavailable in configured PHP version" issue when the
/// configured version is outside the metadata's `Mago\AvailableSince` /
/// `Mago\AvailableUntil` window.
///
/// `display_name` is rendered verbatim into the message — pass the user's
/// original casing (e.g. `Foo::bar`) rather than the lowercased lookup key.
pub fn check_class_like_availability<A>(
    context: &mut Context<'_, '_, A>,
    metadata: &ClassLikeMetadata,
    display_name: &(impl std::fmt::Display + ?Sized),
    span: Span,
) where
    A: Arena,
{
    let kind = match metadata.kind {
        SymbolKind::Class => "class",
        SymbolKind::Interface => "interface",
        SymbolKind::Trait => "trait",
        SymbolKind::Enum => "enum",
    };

    check_availability(
        context,
        IssueCode::UnavailableClassLike,
        kind,
        &metadata.version_constraint,
        display_name,
        span,
    );
}

pub fn check_function_availability<A>(
    context: &mut Context<'_, '_, A>,
    metadata: &FunctionLikeMetadata,
    display_name: &(impl std::fmt::Display + ?Sized),
    span: Span,
) where
    A: Arena,
{
    check_availability(
        context,
        IssueCode::UnavailableFunction,
        "function",
        &metadata.version_constraint,
        display_name,
        span,
    );
}

pub fn check_method_availability<A>(
    context: &mut Context<'_, '_, A>,
    metadata: &FunctionLikeMetadata,
    display_name: &(impl std::fmt::Display + ?Sized),
    span: Span,
) where
    A: Arena,
{
    check_availability(
        context,
        IssueCode::UnavailableMethod,
        "method",
        &metadata.version_constraint,
        display_name,
        span,
    );
}

pub fn check_property_availability<A>(
    context: &mut Context<'_, '_, A>,
    metadata: &PropertyMetadata,
    display_name: &(impl std::fmt::Display + ?Sized),
    span: Span,
) where
    A: Arena,
{
    check_availability(
        context,
        IssueCode::UnavailableProperty,
        "property",
        &metadata.version_constraint,
        display_name,
        span,
    );
}

pub fn check_constant_availability<A>(
    context: &mut Context<'_, '_, A>,
    metadata: &ConstantMetadata,
    display_name: &(impl std::fmt::Display + ?Sized),
    span: Span,
) where
    A: Arena,
{
    check_availability(
        context,
        IssueCode::UnavailableConstant,
        "constant",
        &metadata.version_constraint,
        display_name,
        span,
    );
}

pub fn check_class_constant_availability<A>(
    context: &mut Context<'_, '_, A>,
    metadata: &ClassLikeConstantMetadata,
    display_name: &(impl std::fmt::Display + ?Sized),
    span: Span,
) where
    A: Arena,
{
    check_availability(
        context,
        IssueCode::UnavailableClassConstant,
        "class constant",
        &metadata.version_constraint,
        display_name,
        span,
    );
}

pub fn check_enum_case_availability<A>(
    context: &mut Context<'_, '_, A>,
    metadata: &EnumCaseMetadata,
    display_name: &(impl std::fmt::Display + ?Sized),
    span: Span,
) where
    A: Arena,
{
    check_availability(
        context,
        IssueCode::UnavailableEnumCase,
        "enum case",
        &metadata.version_constraint,
        display_name,
        span,
    );
}

#[allow(clippy::too_many_arguments)]
fn report_unavailable<A>(
    context: &mut Context<'_, '_, A>,
    code: IssueCode,
    kind: &str,
    display_name: &(impl std::fmt::Display + ?Sized),
    constraint: &VersionConstraint,
    configured: PHPVersion,
    span: Span,
) where
    A: Arena,
{
    let configured_str = format_version(configured);

    let availability = describe_availability(constraint);

    let message = format!("The {kind} `{display_name}` is not available in PHP {configured_str}.");

    let mut issue = Issue::error(message).with_annotation(
        Annotation::primary(span).with_message(format!("`{display_name}` is unavailable in PHP {configured_str}.")),
    );

    if !availability.is_empty() {
        issue = issue.with_note(availability);
    }

    issue = issue.with_help(format!(
        "Either raise the configured PHP version to a release that ships this {kind}, or replace the use with a compatible alternative."
    ));

    context.collector.report_with_code(code, issue);
}

fn describe_availability(constraint: &VersionConstraint) -> String {
    match constraint.ranges.as_slice() {
        [] => String::new(),
        [range] => describe_range(range),
        many => {
            let parts: Vec<String> = many.iter().map(describe_range_phrase).collect();
            format!("Available in: {}.", parts.join("; "))
        }
    }
}

/// Standalone sentence form, used when the constraint has exactly one range.
fn describe_range(range: &PHPVersionRange) -> String {
    match (range.min, range.max) {
        (Some(since), Some(until)) => {
            format!("Available between PHP {} and PHP {}.", format_version(since), format_version(until))
        }
        (Some(since), None) => format!("Available since PHP {}.", format_version(since)),
        (None, Some(until)) => format!("Available up to and including PHP {}.", format_version(until)),
        (None, None) => String::new(),
    }
}

/// Phrase form, used when listing multiple ranges in a single sentence.
fn describe_range_phrase(range: &PHPVersionRange) -> String {
    match (range.min, range.max) {
        (Some(since), Some(until)) => format!("PHP {} to {}", format_version(since), format_version(until)),
        (Some(since), None) => format!("PHP {} and later", format_version(since)),
        (None, Some(until)) => format!("up to PHP {}", format_version(until)),
        (None, None) => "all versions".to_string(),
    }
}

/// Formats a [`PHPVersion`] as `MAJOR.MINOR.PATCH`, trimming a trailing `.0`
/// for the common "8.4" case.
fn format_version(version: PHPVersion) -> String {
    if version.patch() == 0 {
        format!("{}.{}", version.major(), version.minor())
    } else {
        format!("{}.{}.{}", version.major(), version.minor(), version.patch())
    }
}
