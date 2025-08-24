use mago_atom::Atom;
use mago_atom::atom;
use mago_codex::ttype::TType;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

/// Represents the result of resolving a member or constant selector.
///
/// A selector is the part after `->`, `?->`, or `::` in an access like
/// `$object->property`, `MyClass::CONSTANT`, or dynamic variants like `$obj->{$propName}`.
/// This enum captures all possible outcomes of statically analyzing that selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedSelector {
    /// A resolved name from a direct identifier (e.g., `foo`).
    Identifier(Atom),
    /// A resolved name from an expression that evaluated to a literal string (e.g., `{'foo'}`).
    LiteralString(Atom),
    /// The selector is a non-literal string; its specific name is unknown statically.
    GenericString,
    /// The selector is of type `mixed`; it might be a valid name at runtime.
    Mixed,
    /// The selector is definitively invalid (e.g., its type is not a string or string-compatible).
    Invalid,
}

impl ResolvedSelector {
    /// Returns the specific name of the selector, if it could be resolved.
    pub fn name(&self) -> Option<Atom> {
        match self {
            Self::Identifier(name) | Self::LiteralString(name) => Some(*name),
            _ => None,
        }
    }

    /// Returns `true` if the selector's value could not be determined statically.
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Self::GenericString | Self::Mixed)
    }
}

/// An internal enum to distinguish between member and constant selectors for error reporting.
#[derive(Clone, Copy)]
enum SelectorKind {
    Member,
    Constant,
}

impl SelectorKind {
    fn as_str(&self) -> &'static str {
        match self {
            SelectorKind::Member => "member",
            SelectorKind::Constant => "constant",
        }
    }
}

/// Resolves the selector part of a class member access (property or method).
///
/// This handles selectors in expressions like `$object->member` or `$object->{$var}`.
pub fn resolve_member_selector<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    selector: &ClassLikeMemberSelector<'arena>,
) -> Result<Vec<ResolvedSelector>, AnalysisError> {
    match selector {
        ClassLikeMemberSelector::Identifier(ident) => Ok(vec![ResolvedSelector::Identifier(atom(ident.value))]),
        ClassLikeMemberSelector::Expression(expr) => {
            let was_inside_general_use = block_context.inside_general_use;
            block_context.inside_general_use = true;
            expr.expression.analyze(context, block_context, artifacts)?;
            block_context.inside_general_use = was_inside_general_use;

            let selector_type = artifacts.get_expression_type(&expr.expression);

            Ok(resolve_selector_from_type(context, selector_type, expr.span(), SelectorKind::Member))
        }
        ClassLikeMemberSelector::Variable(var) => {
            let was_inside_general_use = block_context.inside_general_use;
            block_context.inside_general_use = true;
            var.analyze(context, block_context, artifacts)?;
            block_context.inside_general_use = was_inside_general_use;

            let selector_type = artifacts.get_expression_type(&var);

            Ok(resolve_selector_from_type(context, selector_type, var.span(), SelectorKind::Member))
        }
    }
}

/// Resolves the selector part of a class constant access.
///
/// This handles selectors in expressions like `ClassName::CONSTANT` or `ClassName::{$var}`.
pub fn resolve_constant_selector<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    selector: &ClassLikeConstantSelector<'arena>,
) -> Result<Vec<ResolvedSelector>, AnalysisError> {
    match selector {
        ClassLikeConstantSelector::Identifier(ident) => Ok(vec![ResolvedSelector::Identifier(atom(ident.value))]),
        ClassLikeConstantSelector::Expression(expr) => {
            let was_inside_general_use = block_context.inside_general_use;
            block_context.inside_general_use = true;
            expr.expression.analyze(context, block_context, artifacts)?;
            block_context.inside_general_use = was_inside_general_use;

            let selector_type = artifacts.get_expression_type(&expr.expression);

            Ok(resolve_selector_from_type(context, selector_type, expr.span(), SelectorKind::Constant))
        }
    }
}

/// Analyzes the type of a selector expression to determine the resolved selector name(s).
fn resolve_selector_from_type(
    context: &mut Context,
    selector_type: Option<&TUnion>,
    selector_span: Span,
    kind: SelectorKind,
) -> Vec<ResolvedSelector> {
    let kind_str = kind.as_str();

    let Some(selector_type) = selector_type else {
        let issue_kind = match kind {
            SelectorKind::Constant => IssueCode::UnknownConstantSelectorType,
            SelectorKind::Member => IssueCode::UnknownMemberSelectorType,
        };

        context.collector.report_with_code(
            issue_kind,
            Issue::error(format!("Cannot determine the type of the expression used as a {kind_str} selector."))
                .with_annotation(Annotation::primary(selector_span).with_message("The type of this expression is unknown"))
                .with_help(format!("Ensure the expression has a resolvable type (typically `string`), or use a literal {kind_str} name.")),
        );

        return vec![ResolvedSelector::Invalid];
    };

    if selector_type.is_never() {
        return vec![ResolvedSelector::Invalid];
    }

    let mut resolved_selectors = vec![];
    for atomic in selector_type.types.as_ref() {
        if let Some(literal_string) = atomic.get_literal_string_value() {
            resolved_selectors.push(ResolvedSelector::LiteralString(atom(literal_string)));
            continue;
        }

        let atomic_type_id = atomic.get_id();

        if atomic.is_any_string() {
            let issue_kind = match kind {
                SelectorKind::Constant => IssueCode::StringConstantSelector,
                SelectorKind::Member => IssueCode::StringMemberSelector,
            };

            context.collector.report_with_code(
                issue_kind,
                Issue::warning(format!("This {kind_str} selector uses a non-literal string type (`{atomic_type_id}`); its specific value cannot be statically determined."))
                    .with_annotation(Annotation::primary(selector_span).with_message(format!("This expression (type `{atomic_type_id}`) provides the {kind_str} name")))
                    .with_note("While this may work at runtime, its existence and type cannot be checked statically.".to_string())
            );

            resolved_selectors.push(ResolvedSelector::GenericString);
        } else {
            let issue_kind = match kind {
                SelectorKind::Constant => IssueCode::InvalidConstantSelector,
                SelectorKind::Member => IssueCode::InvalidMemberSelector,
            };

            context.collector.report_with_code(
                issue_kind,
                Issue::error(format!(
                    "Invalid type `{atomic_type_id}` used as a {kind_str} selector; a string is expected."
                ))
                .with_annotation(
                    Annotation::primary(selector_span)
                        .with_message(format!("This expression has type `{atomic_type_id}`")),
                ),
            );

            if atomic.is_mixed() {
                resolved_selectors.push(ResolvedSelector::Mixed);
            } else {
                resolved_selectors.push(ResolvedSelector::Invalid);
            }
        }
    }

    resolved_selectors
}
