//! Checks for unused private/protected class members (properties and methods).
//!
//! This module detects properties and methods that are declared but never used:
//! - Private members in any class
//! - Protected members in final classes (since they can't be accessed by subclasses)
//!
//! It also performs transitive analysis: if a member is only referenced by other
//! unused members, it is also considered unused.
//!
//! Members with names starting with underscore (`_`) are considered intentionally unused
//! and are not reported. This also covers magic methods (`__construct`, `__call`, etc.)
//! since they all start with double underscore.

use ahash::HashSet;

use mago_atom::Atom;
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

/// Represents a checkable member (property or method) with its metadata.
#[derive(Clone)]
struct CheckableMember {
    symbol_id: SymbolIdentifier,
    span: Span,
    is_property: bool,
}

/// Check for unused members (properties and methods) with transitive analysis.
///
/// This function detects:
/// 1. Members with no references (directly unused)
/// 2. Members whose only references come from other unused members (transitively unused)
///
/// Reports private members (and protected members in final classes) that are unused.
/// Returns the set of unused member symbol IDs for use by other checks.
pub fn check_unused_members_with_transitivity<'ctx, 'arena>(
    class_name: Atom,
    class_span: Span,
    class_like_metadata: &ClassLikeMetadata,
    symbol_references: &SymbolReferences,
    context: &mut Context<'ctx, 'arena>,
) -> HashSet<SymbolIdentifier> {
    if class_like_metadata.kind.is_trait() {
        return HashSet::default();
    }

    if class_like_metadata.kind.is_interface() {
        return HashSet::default();
    }

    let is_final = class_like_metadata.flags.is_final() || class_like_metadata.kind.is_enum();
    let mut checkable_members: Vec<CheckableMember> = Vec::new();

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

        if !property.read_visibility.is_private()
            && class_like_metadata.overridden_property_ids.contains_key(property_name)
        {
            continue;
        }

        if let Some(property_span) = property.name_span.or(property.span) {
            checkable_members.push(CheckableMember {
                symbol_id: (class_name, *property_name),
                span: property_span,
                is_property: true,
            });
        }
    }

    for method_name in &class_like_metadata.methods {
        if let Some(declaring_method_id) = class_like_metadata.declaring_method_ids.get(method_name)
            && *declaring_method_id.get_class_name() != class_name
        {
            continue;
        }

        let Some(method_metadata) = context.codebase.function_likes.get(&(class_name, *method_name)) else {
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

        if class_like_metadata
            .used_traits
            .iter()
            .any(|trait_name| context.codebase.method_exists(trait_name, method_name))
        {
            // non-abstract trait method override, could be used in the trait itself.
            continue;
        }

        let method_span = method_metadata.name_span.unwrap_or(method_metadata.span);
        checkable_members.push(CheckableMember {
            symbol_id: (class_name, *method_name),
            span: method_span,
            is_property: false,
        });
    }

    if checkable_members.is_empty() {
        return HashSet::default();
    }

    let mut unused_members: HashSet<SymbolIdentifier> = HashSet::default();
    for member in &checkable_members {
        if !is_member_referenced(symbol_references, &member.symbol_id) {
            unused_members.insert(member.symbol_id);
        }
    }

    let checkable_set: HashSet<SymbolIdentifier> = checkable_members.iter().map(|m| m.symbol_id).collect();
    loop {
        let mut newly_unused: HashSet<SymbolIdentifier> = HashSet::default();

        for member in &checkable_members {
            if unused_members.contains(&member.symbol_id) {
                continue;
            }

            if all_references_from_unused(symbol_references, &member.symbol_id, &unused_members, &checkable_set) {
                newly_unused.insert(member.symbol_id);
            }
        }

        if newly_unused.is_empty() {
            break;
        }

        unused_members.extend(newly_unused);
    }

    for member in &checkable_members {
        if unused_members.contains(&member.symbol_id) {
            if member.is_property {
                report_unused_property(context, class_span, member.symbol_id.1, member.span);
            } else {
                report_unused_method(context, class_span, member.symbol_id.1, member.span);
            }
        }
    }

    unused_members
}

/// Checks if all references to a member come from unused members.
///
/// Returns true if:
/// - The member has no references, OR
/// - All referencing symbols are in the unused set
fn all_references_from_unused(
    symbol_references: &SymbolReferences,
    symbol_id: &SymbolIdentifier,
    unused_members: &HashSet<SymbolIdentifier>,
    checkable_set: &HashSet<SymbolIdentifier>,
) -> bool {
    let referencing_symbols = symbol_references.get_references_to_symbol(*symbol_id);

    if referencing_symbols.is_empty() {
        return true;
    }

    for referencing in referencing_symbols {
        if checkable_set.contains(referencing) {
            if !unused_members.contains(referencing) {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

/// Check for write-only properties in a class-like.
///
/// Reports private properties (and protected properties in final classes)
/// that are written to but never read.
///
/// The `unused_members` set contains property symbols that were already reported
/// as unused (via transitive analysis) and should be skipped.
pub fn check_write_only_properties<'ctx, 'arena>(
    class_name: Atom,
    class_span: Span,
    class_like_metadata: &ClassLikeMetadata,
    symbol_references: &SymbolReferences,
    unused_members: &HashSet<SymbolIdentifier>,
    context: &mut Context<'ctx, 'arena>,
) {
    if class_like_metadata.kind.is_trait() {
        return;
    }

    if class_like_metadata.kind.is_interface() {
        return;
    }

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
        if unused_members.contains(&symbol_id) {
            continue;
        }

        if !is_member_referenced(symbol_references, &symbol_id) {
            continue;
        }

        let has_reads = symbol_references.count_property_reads(&symbol_id) > 0;
        let has_writes = symbol_references.count_property_writes(&symbol_id) > 0;

        if has_writes
            && !has_reads
            && let Some(property_span) = property.name_span.or(property.span)
        {
            report_write_only_property(context, class_span, *property_name, property_span);
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

/// Reports a write-only property.
fn report_write_only_property<'arena>(
    context: &mut Context<'_, 'arena>,
    class_span: Span,
    property_name: Atom,
    property_span: Span,
) {
    let issue = Issue::help(format!("Property `{property_name}` is written to but never read."))
        .with_code(IssueCode::WriteOnlyProperty)
        .with_annotations([
            Annotation::primary(property_span).with_message(format!("Property `{property_name}` is declared here.")),
            Annotation::secondary(class_span),
        ])
        .with_note("This property is assigned values but its value is never read within the class.")
        .with_help(
            "Consider removing the property if its value is not needed, or read the value somewhere if it should be used.",
        );

    context.collector.report(issue);
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
