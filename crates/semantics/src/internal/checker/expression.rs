use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Access;
use mago_syntax::ast::Call;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::Instantiation;
use mago_syntax::ast::UnaryPrefixOperator;

use crate::internal::context::Context;

#[inline]
pub fn check_for_new_without_parenthesis(object_expr: &Expression, context: &mut Context<'_, '_, '_>, operation: &str) {
    if context.version.is_supported(Feature::NewWithoutParentheses) {
        return;
    }

    let Expression::Instantiation(instantiation) = object_expr else {
        return;
    };

    context.report(
        Issue::error(format!(
            "Direct {operation} on `new` expressions without parentheses is only available in PHP 8.4 and above."
        ))
        .with_annotation(
            Annotation::primary(instantiation.span())
                .with_message(format!("Unparenthesized `new` expression used for {operation}.")),
        ),
    );
}

#[inline]
pub fn check_instantiation_class_reference(instantiation: &Instantiation, context: &mut Context<'_, '_, '_>) {
    if let Some((span, operator)) = invalid_class_reference_break(instantiation.class) {
        context.report(
            Issue::error(format!("`{operator}` cannot follow a class name in a `new` expression."))
                .with_annotation(Annotation::primary(span).with_message(format!("`{operator}` not allowed here")))
                .with_annotation(Annotation::secondary(instantiation.new.span).with_message("`new` starts here"))
                .with_note("Only variable roots may extend with member access; class names terminate.")
                .with_help("Wrap the new expression in parentheses, e.g. `(new Foo())->bar()`."),
        );
    }
}

fn invalid_class_reference_break(class: &Expression) -> Option<(Span, &'static str)> {
    match class {
        Expression::Identifier(_)
        | Expression::ConstantAccess(_)
        | Expression::Self_(_)
        | Expression::Parent(_)
        | Expression::Static(_)
        | Expression::Parenthesized(_)
        | Expression::Variable(_) => None,
        Expression::Access(Access::StaticProperty(_)) => None,
        Expression::Access(Access::ClassConstant(c)) => Some((c.double_colon, "::")),
        Expression::Access(Access::Property(p)) => is_new_variable_chain(p.object).then(|| (p.arrow, "->")),
        Expression::Access(Access::NullSafeProperty(p)) => {
            is_new_variable_chain(p.object).then(|| (p.question_mark_arrow, "?->"))
        }
        Expression::ArrayAccess(a) => is_new_variable_chain(a.array).then(|| (a.left_bracket, "[")),
        other => Some((other.span(), "this expression")),
    }
}

const fn is_new_variable_chain(expr: &Expression) -> bool {
    match expr {
        Expression::Variable(_) => true,
        Expression::Access(Access::StaticProperty(_)) => true,
        Expression::Access(Access::Property(p)) => is_new_variable_chain(p.object),
        Expression::Access(Access::NullSafeProperty(p)) => is_new_variable_chain(p.object),
        Expression::ArrayAccess(a) => is_new_variable_chain(a.array),
        _ => false,
    }
}

#[inline]
pub fn check_for_clone_with(expr: &Expression, context: &mut Context<'_, '_, '_>) {
    if context.version.is_supported(Feature::CloneWith) {
        return;
    }

    let Expression::Call(Call::Function(FunctionCall { function, argument_list })) = expr else {
        return;
    };

    let Expression::Identifier(clone_ident) = function else {
        return;
    };

    if !clone_ident.value().eq_ignore_ascii_case(b"clone") {
        return;
    }

    if argument_list.arguments.len() <= 1 {
        return;
    }

    context.report(
        Issue::error("Cloning with properties is only available in PHP 8.5 and above.".to_string())
        .with_annotation(
            Annotation::primary(clone_ident.span())
                .with_message("Clone with properties used here."),
        )
        .with_note(
            "Consider using a standard clone operation without additional properties for compatibility with earlier PHP versions.",
        ).with_help(
            "Upgrade to PHP 8.5 or above to use cloning with properties.",
        )
    );
}

#[inline]
pub fn check_unary_prefix_operator(unary_prefix_operator: &UnaryPrefixOperator, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::UnsetCast)
        && let UnaryPrefixOperator::UnsetCast(span, _) = unary_prefix_operator
    {
        context.report(
            Issue::error("The `unset` cast is no longer supported in PHP 8.0 and later.")
                .with_annotation(Annotation::primary(*span).with_message("Unset cast used here.")),
        );
    }

    if !context.version.is_supported(Feature::VoidCast)
        && let UnaryPrefixOperator::VoidCast(span, _) = unary_prefix_operator
    {
        context.report(
            Issue::error("The `void` cast is only available in PHP 8.5 and later.")
                .with_annotation(Annotation::primary(*span).with_message("Void cast used here.")),
        );
    }
}
