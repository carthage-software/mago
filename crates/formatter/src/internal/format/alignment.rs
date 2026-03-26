//! Alignment utilities for consecutive assignment-like constructs.
//!
//! This module provides functionality for aligning consecutive assignment-like
//! constructs such as variable assignments, class properties, constants, and
//! array key-value pairs.

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ClassLikeConstant;
use mago_syntax::ast::ClassLikeMember;
use mago_syntax::ast::EnumCaseItem;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Modifier;
use mago_syntax::ast::Property;
use mago_syntax::ast::PropertyItem;
use mago_syntax::ast::Sequence;
use mago_syntax::ast::Statement;

use crate::internal::FormatterState;
use crate::internal::format::misc::has_new_line_in_range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignableKind {
    VariableAssignment,
    PropertyWithDefault,
    ClassConstant,
    GlobalConstant,
    EnumBackedCase,
}

/// Alignment widths for formatting.
#[derive(Debug, Clone, Copy, Default)]
pub struct AlignmentWidths {
    /// Width of the type hint column (for properties only).
    /// This is used to align variable names when types have different lengths.
    pub type_width: usize,
    /// Width of the name/key column.
    pub name_width: usize,
}

impl AlignmentWidths {
    /// Create new alignment widths.
    pub const fn new(name_width: usize) -> Self {
        Self { type_width: 0, name_width }
    }

    /// Create alignment widths with type width (for properties).
    pub const fn with_type_width(type_width: usize, name_width: usize) -> Self {
        Self { type_width, name_width }
    }
}

/// A run of consecutive alignable items with their calculated widths.
#[derive(Debug)]
pub struct AlignmentRun {
    /// Start index of the run (inclusive).
    pub start: usize,
    /// End index of the run (exclusive).
    pub end: usize,
    /// Calculated alignment widths for this run.
    pub widths: AlignmentWidths,
}

impl AlignmentRun {
    /// Create a new alignment run.
    pub const fn new(start: usize, end: usize, widths: AlignmentWidths) -> Self {
        Self { start, end, widths }
    }

    /// Check if an index falls within this run.
    pub const fn contains(&self, index: usize) -> bool {
        index >= self.start && index < self.end
    }
}

/// Check if there's a blank line between two spans.
pub fn has_blank_line_between(f: &FormatterState<'_, '_>, prev_span: Span, next_span: Span) -> bool {
    let source = f.source_text;
    let start = prev_span.end_offset() as usize;
    let end = next_span.start_offset() as usize;

    if start >= end || end > source.len() {
        return false;
    }

    let between = &source[start..end];
    let newline_count = between.chars().filter(|&c| c == '\n').count();

    newline_count >= 2
}

/// Check if there's a comment between two spans.
pub fn has_comment_between(f: &FormatterState<'_, '_>, prev_span: Span, next_span: Span) -> bool {
    f.all_comments.iter().any(|comment| {
        let comment_span = comment.span;
        comment_span.start_offset() >= prev_span.end_offset() && comment_span.end_offset() <= next_span.start_offset()
    })
}

/// Check if a property is a plain property with a default value.
/// Hooked properties are excluded from alignment because they span multiple lines.
fn is_plain_property_with_default(prop: &Property<'_>) -> bool {
    match prop {
        Property::Plain(p) => p.items.iter().any(|item| matches!(item, PropertyItem::Concrete(_))),
        Property::Hooked(_) => false,
    }
}

/// Get the attribute lists for a property.
fn property_has_attributes(prop: &Property<'_>) -> bool {
    match prop {
        Property::Plain(p) => !p.attribute_lists.is_empty(),
        Property::Hooked(p) => !p.attribute_lists.is_empty(),
    }
}

/// Check if two members have compatible modifiers for alignment.
/// Members with different modifier combinations should not be aligned together.
fn have_compatible_modifiers(prev: &ClassLikeMember<'_>, curr: &ClassLikeMember<'_>) -> bool {
    match (prev, curr) {
        (ClassLikeMember::Property(prev_prop), ClassLikeMember::Property(curr_prop)) => {
            get_property_modifier_signature(prev_prop) == get_property_modifier_signature(curr_prop)
        }
        (ClassLikeMember::Constant(prev_const), ClassLikeMember::Constant(curr_const)) => {
            get_constant_modifier_signature(prev_const) == get_constant_modifier_signature(curr_const)
        }
        (ClassLikeMember::EnumCase(_), ClassLikeMember::EnumCase(_)) => true,
        _ => false,
    }
}

/// Get a signature representing the modifiers of a property.
fn get_property_modifier_signature(prop: &Property<'_>) -> u32 {
    let modifiers = match prop {
        Property::Plain(p) => &p.modifiers,
        Property::Hooked(p) => &p.modifiers,
    };
    calculate_modifier_signature(modifiers)
}

/// Get a signature representing the modifiers of a constant.
fn get_constant_modifier_signature(constant: &ClassLikeConstant<'_>) -> u32 {
    calculate_modifier_signature(&constant.modifiers)
}

/// Calculate a numeric signature from modifiers.
/// Each modifier type gets a bit in the signature.
fn calculate_modifier_signature(modifiers: &Sequence<'_, Modifier<'_>>) -> u32 {
    let mut sig = 0u32;
    for modifier in modifiers.iter() {
        sig |= match modifier {
            Modifier::Public(_) => 1 << 0,
            Modifier::Protected(_) => 1 << 1,
            Modifier::Private(_) => 1 << 2,
            Modifier::Static(_) => 1 << 3,
            Modifier::Final(_) => 1 << 4,
            Modifier::Abstract(_) => 1 << 5,
            Modifier::Readonly(_) => 1 << 6,
            Modifier::PublicSet(_) => 1 << 7,
            Modifier::ProtectedSet(_) => 1 << 8,
            Modifier::PrivateSet(_) => 1 << 9,
        };
    }
    sig
}

/// Detect alignment runs in a slice of class-like members.
///
/// Returns a list of runs where consecutive members of the same type can be aligned.
/// Different member types (properties vs constants) break alignment runs.
pub fn detect_class_member_alignment_runs<'arena>(
    f: &FormatterState<'_, 'arena>,
    members: &'arena [ClassLikeMember<'arena>],
) -> Vec<AlignmentRun> {
    if !f.settings.align_assignment_like || members.is_empty() {
        return Vec::new();
    }

    let mut runs = Vec::new();
    let mut run_start: Option<(usize, AlignableKind)> = None;

    for (i, member) in members.iter().enumerate() {
        let kind = match member {
            ClassLikeMember::Property(prop) => {
                if is_plain_property_with_default(prop) {
                    Some(AlignableKind::PropertyWithDefault)
                } else {
                    None
                }
            }
            ClassLikeMember::Constant(_) => Some(AlignableKind::ClassConstant),
            ClassLikeMember::EnumCase(case) => {
                if matches!(case.item, EnumCaseItem::Backed(_)) {
                    Some(AlignableKind::EnumBackedCase)
                } else {
                    None
                }
            }
            _ => None,
        };

        let should_break_run = run_start.is_some_and(|(_start_idx, start_kind)| {
            if kind != Some(start_kind) {
                return true;
            }

            if i > 0 {
                let prev_span = members[i - 1].span();
                let curr_span = member.span();
                if has_blank_line_between(f, prev_span, curr_span) || has_comment_between(f, prev_span, curr_span) {
                    return true;
                }
            }

            let has_attributes = match member {
                ClassLikeMember::Property(prop) => property_has_attributes(prop),
                ClassLikeMember::Constant(c) => !c.attribute_lists.is_empty(),
                ClassLikeMember::EnumCase(c) => !c.attribute_lists.is_empty(),
                _ => false,
            };

            if has_attributes {
                return true;
            }

            if i > 0 && !have_compatible_modifiers(&members[i - 1], member) {
                return true;
            }

            false
        });

        if should_break_run {
            if let Some((start_idx, start_kind)) = run_start
                && i - start_idx >= 2
            {
                let widths = calculate_class_member_widths(&members[start_idx..i], start_kind);
                runs.push(AlignmentRun::new(start_idx, i, widths));
            }
            run_start = None;
        }

        if let Some(k) = kind {
            if run_start.is_none() {
                run_start = Some((i, k));
            }
        } else {
            // Non-alignable member breaks any run
            if let Some((start_idx, start_kind)) = run_start
                && i - start_idx >= 2
            {
                let widths = calculate_class_member_widths(&members[start_idx..i], start_kind);
                runs.push(AlignmentRun::new(start_idx, i, widths));
            }
            run_start = None;
        }
    }

    // Flush any remaining run
    if let Some((start_idx, start_kind)) = run_start {
        let len = members.len();
        if len - start_idx >= 2 {
            let widths = calculate_class_member_widths(&members[start_idx..], start_kind);
            runs.push(AlignmentRun::new(start_idx, len, widths));
        }
    }

    runs
}

/// Calculate alignment widths for a slice of class-like members.
fn calculate_class_member_widths(members: &[ClassLikeMember<'_>], kind: AlignableKind) -> AlignmentWidths {
    let mut max_type_width = 0usize;
    let mut max_name_width = 0usize;

    for member in members {
        match (kind, member) {
            (AlignableKind::PropertyWithDefault, ClassLikeMember::Property(prop)) => {
                let (type_width, name_width) = calculate_plain_property_widths(prop);
                max_type_width = max_type_width.max(type_width);
                max_name_width = max_name_width.max(name_width);
            }
            (AlignableKind::ClassConstant, ClassLikeMember::Constant(constant)) => {
                for item in constant.items.iter() {
                    let name_width = item.name.value.len();
                    max_name_width = max_name_width.max(name_width);
                }
            }
            (AlignableKind::EnumBackedCase, ClassLikeMember::EnumCase(case)) => {
                let name_width = case.item.name().value.len();
                max_name_width = max_name_width.max(name_width);
            }
            _ => {}
        }
    }

    if kind == AlignableKind::PropertyWithDefault {
        AlignmentWidths::with_type_width(max_type_width, max_name_width)
    } else {
        AlignmentWidths::new(max_name_width)
    }
}

/// Calculate the type width and name width for a plain property.
/// Type width is the width of the type hint (e.g., "int" = 3, "string" = 6).
/// Name width is the width of the variable name (e.g., "$foo" = 4).
fn calculate_plain_property_widths(prop: &Property<'_>) -> (usize, usize) {
    match prop {
        Property::Plain(p) => {
            let type_width = p.hint.as_ref().map_or(0, |h| (h.span().end_offset() - h.span().start_offset()) as usize);
            let name_width = p.items.iter().map(|item| item.variable().name.len()).max().unwrap_or(0);
            (type_width, name_width)
        }
        Property::Hooked(_) => (0, 0),
    }
}

/// Get alignment widths for a specific class member index.
pub fn get_member_alignment(runs: &[AlignmentRun], index: usize) -> Option<AlignmentWidths> {
    runs.iter().find(|run| run.contains(index)).map(|run| run.widths)
}

/// Get alignment widths for a specific statement index.
pub fn get_statement_alignment(runs: &[AlignmentRun], index: usize) -> Option<AlignmentWidths> {
    runs.iter().find(|run| run.contains(index)).map(|run| run.widths)
}

/// Detect alignment runs in a slice of statement references.
///
/// This is a variant of `detect_statement_alignment_runs` that works with
/// `&[&Statement]` instead of `&[Statement]`.
pub fn detect_statement_ref_alignment_runs<'arena>(
    f: &FormatterState<'_, 'arena>,
    statements: &[&'arena Statement<'arena>],
) -> Vec<AlignmentRun> {
    if !f.settings.align_assignment_like || statements.is_empty() {
        return Vec::new();
    }

    let mut runs = Vec::new();
    let mut run_start: Option<(usize, AlignableKind)> = None;

    for (i, stmt) in statements.iter().enumerate() {
        let kind = match stmt {
            Statement::Expression(expr_stmt) => {
                if let Expression::Assignment(assign) = &expr_stmt.expression {
                    if matches!(
                        assign.lhs,
                        Expression::Variable(_) | Expression::Access(_) | Expression::ArrayAccess(_)
                    ) {
                        Some(AlignableKind::VariableAssignment)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Statement::Constant(_) => Some(AlignableKind::GlobalConstant),
            _ => None,
        };

        let should_break_run = run_start.is_some_and(|(_start_idx, start_kind)| {
            if kind != Some(start_kind) {
                return true;
            }

            if i > 0 {
                let prev_span = statements[i - 1].span();
                let curr_span = stmt.span();
                if has_blank_line_between(f, prev_span, curr_span) || has_comment_between(f, prev_span, curr_span) {
                    return true;
                }
            }

            false
        });

        if should_break_run {
            if let Some((start_idx, start_kind)) = run_start
                && i - start_idx >= 2
            {
                let widths = calculate_statement_ref_widths(f, &statements[start_idx..i], start_kind);
                runs.push(AlignmentRun::new(start_idx, i, widths));
            }
            run_start = None;
        }

        if let Some(k) = kind {
            if run_start.is_none() {
                run_start = Some((i, k));
            }
        } else {
            if let Some((start_idx, start_kind)) = run_start
                && i - start_idx >= 2
            {
                let widths = calculate_statement_ref_widths(f, &statements[start_idx..i], start_kind);
                runs.push(AlignmentRun::new(start_idx, i, widths));
            }
            run_start = None;
        }
    }

    if let Some((start_idx, start_kind)) = run_start {
        let len = statements.len();
        if len - start_idx >= 2 {
            let widths = calculate_statement_ref_widths(f, &statements[start_idx..], start_kind);
            runs.push(AlignmentRun::new(start_idx, len, widths));
        }
    }

    runs
}

/// Calculate alignment widths for a slice of statement references.
fn calculate_statement_ref_widths(
    f: &FormatterState<'_, '_>,
    statements: &[&Statement<'_>],
    kind: AlignableKind,
) -> AlignmentWidths {
    let mut max_name_width = 0usize;

    for stmt in statements {
        match (kind, *stmt) {
            (AlignableKind::VariableAssignment, Statement::Expression(expr_stmt)) => {
                if let Expression::Assignment(assign) = &expr_stmt.expression {
                    let lhs_span = assign.lhs.span();
                    if has_new_line_in_range(f.source_text, lhs_span.start_offset(), lhs_span.end_offset()) {
                        continue;
                    }

                    let name_width = (lhs_span.end_offset() - lhs_span.start_offset()) as usize;
                    max_name_width = max_name_width.max(name_width);
                }
            }
            (AlignableKind::GlobalConstant, Statement::Constant(constant)) => {
                for item in constant.items.iter() {
                    let name_width = item.name.value.len();
                    max_name_width = max_name_width.max(name_width);
                }
            }
            _ => {}
        }
    }

    AlignmentWidths::new(max_name_width)
}
