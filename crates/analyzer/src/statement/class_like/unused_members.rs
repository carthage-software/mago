//! Checks for unused private/protected class members (properties and methods).
//!
//! This module detects properties and methods that are declared but never used:
//! - Private members in any class
//! - Protected members in final classes (since they can't be accessed by subclasses)
//!
//! Members with names starting with underscore (`_`) are considered intentionally unused
//! and are not reported. This also covers magic methods (`__construct`, `__call`, etc.)
//! since they all start with double underscore.

use mago_atom::Atom;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::reference::SymbolReferences;
use mago_codex::symbol::SymbolIdentifier;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_text_edit::Safety;
use mago_text_edit::TextEdit;

use crate::code::IssueCode;
use crate::context::Context;

/// Check for unused properties in a class-like.
///
/// Reports private properties (and protected properties in final classes)
/// that are never read or written.
pub fn check_unused_properties<'ctx, 'arena>(
    class_name: Atom,
    class_span: Span,
    class_like_metadata: &ClassLikeMetadata,
    symbol_references: &SymbolReferences,
    context: &mut Context<'ctx, 'arena>,
) {
    if class_like_metadata.kind.is_trait() {
        return;
    }

    if class_like_metadata.kind.is_interface() {
        return;
    }

    // Enums are implicitly final (they cannot be extended), so protected members
    // in enums should be checked just like in final classes
    let is_final = class_like_metadata.flags.is_final() || class_like_metadata.kind.is_enum();

    for (property_name, property) in &class_like_metadata.properties {
        if let Some(declaring_class) = class_like_metadata.declaring_property_ids.get(property_name)
            && *declaring_class != class_name
        {
            continue;
        }

        if property.read_visibility.is_public() {
            continue;
        }

        if property.read_visibility.is_protected() && !is_final {
            continue;
        }

        if property_name.starts_with("$_") {
            continue;
        }

        if !property.hooks.is_empty() {
            continue;
        }

        let symbol_id: SymbolIdentifier = (class_name, *property_name);
        if !is_member_referenced(symbol_references, &symbol_id)
            && let Some(property_span) = property.name_span.or(property.span)
        {
            report_unused_property(context, class_span, *property_name, property_span);
        }
    }
}

/// Check for unused methods in a class-like.
///
/// Reports private methods (and protected methods in final classes)
/// that are never called.
pub fn check_unused_methods<'ctx, 'arena>(
    class_name: Atom,
    class_span: Span,
    class_like_metadata: &ClassLikeMetadata,
    symbol_references: &SymbolReferences,
    codebase: &CodebaseMetadata,
    context: &mut Context<'ctx, 'arena>,
) {
    if class_like_metadata.kind.is_trait() {
        return;
    }

    if class_like_metadata.kind.is_interface() {
        return;
    }

    let is_final = class_like_metadata.flags.is_final() || class_like_metadata.kind.is_enum();

    for method_name in &class_like_metadata.methods {
        if let Some(declaring_method_id) = class_like_metadata.declaring_method_ids.get(method_name)
            && *declaring_method_id.get_class_name() != class_name
        {
            continue;
        }

        let Some(method_metadata) = codebase.function_likes.get(&(class_name, *method_name)) else {
            continue;
        };

        let Some(method_meta) = &method_metadata.method_metadata else {
            continue;
        };

        if method_meta.visibility.is_public() {
            continue;
        }

        if method_meta.visibility.is_protected() && !is_final {
            continue;
        }

        if method_name.starts_with('_') {
            continue;
        }

        if method_meta.is_abstract {
            continue;
        }

        if class_like_metadata.overridden_method_ids.contains_key(method_name) {
            continue;
        }

        let symbol_id: SymbolIdentifier = (class_name, *method_name);
        if !is_member_referenced(symbol_references, &symbol_id) {
            let method_span = method_metadata.name_span.unwrap_or(method_metadata.span);
            report_unused_method(context, class_span, *method_name, method_span);
        }
    }
}

/// Checks if a member (property or method) is referenced anywhere in the codebase.
#[inline(always)]
fn is_member_referenced(symbol_references: &SymbolReferences, symbol_id: &SymbolIdentifier) -> bool {
    symbol_references.count_referencing_symbols(symbol_id, false) > 0
        || symbol_references.count_referencing_symbols(symbol_id, true) > 0
}

/// Reports an unused property.
fn report_unused_property<'arena>(
    context: &mut Context<'_, 'arena>,
    class_span: Span,
    property_name: Atom,
    property_span: Span,
) {
    let issue = Issue::help(format!("Property `{property_name}` is never used."))
        .with_code(IssueCode::UnusedProperty)
        .with_annotations([
            Annotation::primary(property_span).with_message(format!("Property `{property_name}` is declared here.")),
            Annotation::secondary(class_span),
        ])
        .with_note("This property is declared but never read or written within the class.")
        .with_help(
            "Consider prefixing the property with an underscore (`$_`) to indicate that it is intentionally unused, or remove it if it is not needed.",
        );

    context.collector.propose(issue, |edits| {
        edits.push(TextEdit::insert(property_span.start_offset() + 1, "_").with_safety(Safety::PotentiallyUnsafe));
    });
}

/// Reports an unused method.
fn report_unused_method<'arena>(
    context: &mut Context<'_, 'arena>,
    class_span: Span,
    method_name: Atom,
    method_span: Span,
) {
    let issue = Issue::help(format!("Method `{method_name}()` is never used."))
        .with_code(IssueCode::UnusedMethod)
        .with_annotations([
            Annotation::primary(method_span).with_message(format!("Method `{method_name}()` is declared here.")),
            Annotation::secondary(class_span),
        ])
        .with_note("This method is declared but never called within the class.")
        .with_help(
            "Consider prefixing the method with an underscore (`_`) to indicate that it is intentionally unused, or remove it if it is not needed.",
        );

    context.collector.propose(issue, |edits| {
        edits.push(TextEdit::insert(method_span.start_offset(), "_").with_safety(Safety::PotentiallyUnsafe));
    });
}
