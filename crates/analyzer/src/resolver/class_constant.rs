use mago_codex::get_class_like;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::r#enum::TEnum;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_interner::StringIdentifier;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ClassLikeConstantSelector;
use mago_syntax::ast::Expression;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;
use crate::resolver::class_name::ResolutionOrigin;
use crate::resolver::class_name::ResolvedClassname;
use crate::resolver::class_name::resolve_classnames_from_expression;
use crate::resolver::selector::ResolvedSelector;
use crate::resolver::selector::resolve_constant_selector;

/// Represents a successfully resolved class constant or enum case.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ResolvedConstant {
    /// The fully-qualified name of the class where the constant is defined.
    pub fq_class_id: StringIdentifier,
    /// The name of the constant or enum case.
    pub const_name: StringIdentifier,
    /// The type of the constant's value or the enum case itself.
    pub const_type: TUnion,
}

/// Holds the results of a constant resolution attempt.
#[derive(Debug, Default)]
pub struct ConstantResolutionResult {
    /// A list of successfully resolved constants and their types.
    pub constants: Vec<ResolvedConstant>,
    /// Flag indicating if any part of the resolution was ambiguous or dynamic.
    pub has_ambiguous_path: bool,
    /// Flag indicating if any part of the resolution was definitively invalid.
    pub has_invalid_path: bool,
}

/// Resolves all possible class constants from a class expression and a constant selector.
pub fn resolve_class_constants<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    class_expr: &Expression,
    constant_selector: &ClassLikeConstantSelector,
    class_expr_is_analyzed: bool,
) -> Result<ConstantResolutionResult, AnalysisError> {
    let mut result = ConstantResolutionResult::default();

    // 1. Resolve all possible class names from the expression.
    let classnames =
        resolve_classnames_from_expression(context, block_context, artifacts, class_expr, class_expr_is_analyzed)?;

    // 2. Resolve all possible constant names from the selector.
    let selectors = resolve_constant_selector(context, block_context, artifacts, constant_selector)?;

    // 3. Iterate through each combination of class and constant to find valid constants.
    for class_resolution in &classnames {
        if class_resolution.is_possibly_invalid() {
            result.has_ambiguous_path = true;
            if class_resolution.origin == ResolutionOrigin::Invalid {
                result.has_invalid_path = true;
            }

            continue;
        }

        let Some(fq_class_id) = class_resolution.fq_class_id else {
            result.has_ambiguous_path = true;
            report_ambiguous_constant_access(context, class_expr);
            continue;
        };

        for selector_resolution in &selectors {
            if selector_resolution.is_dynamic() {
                result.has_ambiguous_path = true;
                continue;
            }

            let Some(const_name) = selector_resolution.name() else {
                result.has_invalid_path = true;
                continue;
            };

            // Handle `::class` magic constant
            if matches!(selector_resolution, ResolvedSelector::Identifier(_))
                && context.interner.lookup(&const_name).eq_ignore_ascii_case("class")
            {
                if let Some(const_type) = handle_class_magic_constant(
                    context,
                    block_context,
                    artifacts,
                    class_resolution,
                    class_expr,
                    constant_selector,
                ) {
                    result.constants.push(ResolvedConstant { fq_class_id, const_name, const_type });
                } else {
                    result.has_invalid_path = true;
                }

                continue;
            }

            // Handle regular constants and enum cases
            let Some(metadata) = get_class_like(context.codebase, context.interner, &fq_class_id) else {
                result.has_invalid_path = true;
                report_non_existent_class(context, &fq_class_id, class_expr.span());
                continue;
            };

            artifacts.symbol_references.add_reference_to_class_member(
                &block_context.scope,
                (fq_class_id, const_name),
                false,
            );

            if let Some(resolved_const) =
                find_constant_in_class(context, metadata, const_name, class_expr.span(), constant_selector.span())
            {
                result.constants.push(resolved_const);
            } else {
                result.has_invalid_path = true;
            }
        }
    }

    Ok(result)
}

/// Specific handler for the `::class` magic constant.
fn handle_class_magic_constant(
    context: &mut Context,
    block_context: &mut BlockContext,
    artifacts: &mut AnalysisArtifacts,
    class_resolution: &ResolvedClassname,
    class_expr: &Expression,
    selector: &ClassLikeConstantSelector,
) -> Option<TUnion> {
    if matches!(class_resolution.origin, ResolutionOrigin::GenericString) {
        context.buffer.report(
            TypingIssueKind::InvalidClassConstantOnString,
            Issue::error("Cannot use `::class` on an expression of type string.")
                .with_annotation(
                    Annotation::primary(class_expr.span()).with_message("This expression is a string here"),
                )
                .with_annotation(Annotation::secondary(selector.span()).with_message("`::class` used here"))
                .with_note("The `::class` magic constant requires a direct class name or an object instance."),
        );

        return None;
    }

    artifacts.symbol_references.add_reference_to_symbol(
        &block_context.scope,
        class_resolution.fq_class_id.unwrap(),
        false,
    );

    let class_string_type = TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::literal(
        class_resolution.fq_class_id.expect("class-like resolution should yield a valid FQ class ID"),
    )));

    Some(TUnion::new(vec![class_string_type]))
}

/// Finds a constant or enum case by name within a class.
fn find_constant_in_class(
    context: &mut Context,
    metadata: &ClassLikeMetadata,
    const_name: StringIdentifier,
    class_span: Span,
    const_span: Span,
) -> Option<ResolvedConstant> {
    // Check for a defined constant
    if let Some(constant_metadata) = metadata.constants.get(&const_name) {
        let const_type = constant_metadata
            .inferred_type
            .clone()
            .map(wrap_atomic)
            .or_else(|| constant_metadata.type_metadata.clone().map(|s| s.type_union))
            .unwrap_or_else(get_mixed);

        return Some(ResolvedConstant { fq_class_id: metadata.name, const_name, const_type });
    }

    // Check for an enum case
    if metadata.kind.is_enum() && metadata.enum_cases.contains_key(&const_name) {
        let const_type =
            TUnion::new(vec![TAtomic::Object(TObject::Enum(TEnum::new_case(metadata.original_name, const_name)))]);

        return Some(ResolvedConstant { fq_class_id: metadata.name, const_name, const_type });
    }

    // Not found, report error.
    report_undefined_constant(context, metadata, const_name, class_span, const_span);
    None
}

/// Reports an error for a class-like that cannot be found in the codebase.
fn report_non_existent_class(context: &mut Context, class_id: &StringIdentifier, class_span: Span) {
    let class_name_str = context.interner.lookup(class_id);
    context.buffer.report(
        TypingIssueKind::NonExistentClassLike,
        Issue::error(format!("Class, interface, enum, or trait `{class_name_str}` not found."))
            .with_annotation(
                Annotation::primary(class_span)
                    .with_message(format!("`{class_name_str}` is not defined or cannot be found")),
            )
            .with_help(
                "Ensure the name is correct, including its namespace, and that it's properly defined and autoloadable.",
            ),
    );
}

/// Reports an error for a class constant or enum case that is not defined on a given class-like.
fn report_undefined_constant(
    context: &mut Context,
    metadata: &ClassLikeMetadata,
    const_name: StringIdentifier,
    class_span: Span,
    const_span: Span,
) {
    let class_kind_str = metadata.kind.as_str();
    let class_str = context.interner.lookup(&metadata.name);
    let constant_name_str = context.interner.lookup(&const_name);

    let (main_message, primary_annotation_message) = if metadata.kind.is_enum() {
        (
            format!("Undefined enum constant or case: `{class_str}::{constant_name_str}`."),
            format!("Constant or case `{constant_name_str}` not found in `{class_str}`"),
        )
    } else {
        (
            format!("Undefined {class_kind_str} constant: `{class_str}::{constant_name_str}`."),
            format!("Constant `{constant_name_str}` not found in `{class_str}`"),
        )
    };

    context.buffer.report(
        TypingIssueKind::UndefinedClassLikeConstant,
        Issue::error(main_message)
            .with_annotation(
                Annotation::primary(const_span).with_message(primary_annotation_message),
            )
            .with_annotation(
                Annotation::secondary(class_span)
                    .with_message(format!("On this {class_kind_str} `{class_str}`")),
            )
            .with_help(format!(
                "Check for typos or ensure `{constant_name_str}` is defined in `{class_str}` or its ancestors/interfaces.",
            )),
    );
}

/// Reports a warning when a constant is accessed on an ambiguous type like `object` or `class-string`.
fn report_ambiguous_constant_access(context: &mut Context, class_expr: &Expression) {
    context.buffer.report(
        TypingIssueKind::AmbiguousClassLikeConstantAccess,
        Issue::warning("Cannot reliably determine class for constant access due to an ambiguous type.")
            .with_annotation(
                Annotation::primary(class_expr.span())
                    .with_message("This expression does not specify a concrete class"),
            )
            .with_note("To fetch a class constant, the specific class must be known. General types like `object` or a generic `class-string` are too ambiguous for static analysis to verify constant existence.")
            .with_help("Provide a more specific type for the class expression (e.g., `MyClass`), or use `instanceof` checks to narrow it down before accessing constants."),
    );
}
