use mago_database::file::HasFileId;
use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Access;
use mago_syntax::ast::ArrowFunction;
use mago_syntax::ast::Block;
use mago_syntax::ast::Call;
use mago_syntax::ast::Closure;
use mago_syntax::ast::Expression;
use mago_syntax::ast::ForBody;
use mago_syntax::ast::ForeachBody;
use mago_syntax::ast::Function;
use mago_syntax::ast::FunctionLikeParameterList;
use mago_syntax::ast::FunctionLikeReturnTypeHint;
use mago_syntax::ast::Hint;
use mago_syntax::ast::IfBody;
use mago_syntax::ast::Statement;
use mago_syntax::ast::Variable;
use mago_syntax::ast::WhileBody;

use crate::internal::context::Context;

mod parameter;

pub use parameter::*;

use super::returns_generator;

/// Helper function to check if an expression contains $this
fn contains_this_in_expression(expression: &Expression<'_>) -> Option<Span> {
    // Check if this expression is $this
    if let Expression::Variable(Variable::Direct(var)) = expression
        && var.name == "$this"
    {
        return Some(var.span());
    }

    // Don't recurse into nested closures/arrow functions/anonymous classes
    // as they have their own $this binding
    match expression {
        Expression::Closure(_) | Expression::ArrowFunction(_) | Expression::AnonymousClass(_) => {
            return None;
        }
        _ => {}
    }

    // For now, we'll use a simple approach that just checks common cases
    // A more complete implementation would traverse all expression types
    match expression {
        Expression::Binary(binary) => {
            if let Some(span) = contains_this_in_expression(binary.lhs) {
                return Some(span);
            }
            contains_this_in_expression(binary.rhs)
        }
        Expression::Parenthesized(paren) => contains_this_in_expression(paren.expression),
        Expression::Access(access) => match access {
            Access::Property(prop) => contains_this_in_expression(prop.object),
            Access::NullSafeProperty(prop) => contains_this_in_expression(prop.object),
            _ => None,
        },
        Expression::Call(call) => match call {
            Call::Method(method_call) => contains_this_in_expression(method_call.object),
            Call::NullSafeMethod(method_call) => contains_this_in_expression(method_call.object),
            _ => None,
        },
        Expression::Conditional(conditional) => {
            if let Some(span) = contains_this_in_expression(conditional.condition) {
                return Some(span);
            }
            if let Some(then) = &conditional.then
                && let Some(span) = contains_this_in_expression(then)
            {
                return Some(span);
            }
            contains_this_in_expression(conditional.r#else)
        }
        Expression::ArrayAccess(array_access) => contains_this_in_expression(array_access.array),
        _ => None,
    }
}

/// Helper function to check if a block contains $this
fn contains_this_in_block(block: &Block<'_>) -> Option<Span> {
    for statement in &block.statements {
        if let Some(span) = contains_this_in_statement(statement) {
            return Some(span);
        }
    }
    None
}

/// Helper function to check if a statement contains $this
fn contains_this_in_statement(statement: &Statement<'_>) -> Option<Span> {
    match statement {
        Statement::Block(block) => contains_this_in_block(block),
        Statement::Expression(expression) => contains_this_in_expression(expression.expression),
        Statement::Return(r#return) => {
            if let Some(value) = &r#return.value {
                contains_this_in_expression(value)
            } else {
                None
            }
        }
        Statement::Echo(echo) => {
            for expr in &echo.values {
                if let Some(span) = contains_this_in_expression(expr) {
                    return Some(span);
                }
            }
            None
        }
        // For other statement types, we'll need to check their bodies
        Statement::If(r#if) => {
            if let Some(span) = contains_this_in_expression(r#if.condition) {
                return Some(span);
            }
            match &r#if.body {
                IfBody::Statement(stmt_body) => contains_this_in_statement(stmt_body.statement),
                IfBody::ColonDelimited(colon_body) => {
                    for stmt in &colon_body.statements {
                        if let Some(span) = contains_this_in_statement(stmt) {
                            return Some(span);
                        }
                    }
                    None
                }
            }
        }
        Statement::While(r#while) => {
            if let Some(span) = contains_this_in_expression(r#while.condition) {
                return Some(span);
            }
            match &r#while.body {
                WhileBody::Statement(stmt) => contains_this_in_statement(stmt),
                WhileBody::ColonDelimited(colon_body) => {
                    for stmt in &colon_body.statements {
                        if let Some(span) = contains_this_in_statement(stmt) {
                            return Some(span);
                        }
                    }
                    None
                }
            }
        }
        Statement::For(r#for) => {
            for init in &r#for.initializations {
                if let Some(span) = contains_this_in_expression(init) {
                    return Some(span);
                }
            }
            for condition in &r#for.conditions {
                if let Some(span) = contains_this_in_expression(condition) {
                    return Some(span);
                }
            }
            for increment in &r#for.increments {
                if let Some(span) = contains_this_in_expression(increment) {
                    return Some(span);
                }
            }
            match &r#for.body {
                ForBody::Statement(stmt) => contains_this_in_statement(stmt),
                ForBody::ColonDelimited(colon_body) => {
                    for stmt in &colon_body.statements {
                        if let Some(span) = contains_this_in_statement(stmt) {
                            return Some(span);
                        }
                    }
                    None
                }
            }
        }
        Statement::Foreach(foreach) => {
            if let Some(span) = contains_this_in_expression(foreach.expression) {
                return Some(span);
            }
            match &foreach.body {
                ForeachBody::Statement(stmt) => contains_this_in_statement(stmt),
                ForeachBody::ColonDelimited(colon_body) => {
                    for stmt in &colon_body.statements {
                        if let Some(span) = contains_this_in_statement(stmt) {
                            return Some(span);
                        }
                    }
                    None
                }
            }
        }
        Statement::Try(r#try) => contains_this_in_block(&r#try.block),
        _ => None,
    }
}

#[inline]
pub fn check_function<'arena>(function: &Function<'arena>, context: &mut Context<'_, '_, 'arena>) {
    check_for_promoted_properties_outside_constructor(&function.parameter_list, context);
    let Some(return_hint) = &function.return_type_hint else {
        return;
    };

    let name = function.name.value;
    let fqfn = context.get_name(function.name.span.start);

    match &return_hint.hint {
        Hint::Void(_) => {
            for r#return in mago_syntax::utils::find_returns_in_block(&function.body) {
                if let Some(val) = &r#return.value {
                    context.report(
                        Issue::error(format!("Function `{name}` with return type `void` must not return a value."))
                            .with_annotation(Annotation::primary(val.span()).with_message("Return value found here."))
                            .with_annotation(
                                Annotation::secondary(function.span())
                                    .with_message(format!("Function `{fqfn}` defined here.")),
                            )
                            .with_help("Remove the return type hint or the return value."),
                    );
                }
            }
        }
        Hint::Never(_) => {
            for r#return in mago_syntax::utils::find_returns_in_block(&function.body) {
                context.report(
                    Issue::error(format!("Function `{name}` with return type `never` must not return."))
                        .with_annotation(
                            Annotation::primary(r#return.span()).with_message("Return statement found here."),
                        )
                        .with_annotation(
                            Annotation::secondary(function.span())
                                .with_message(format!("Function `{fqfn}` defined here.")),
                        )
                        .with_help("Remove the return type hint or the return statement."),
                );
            }
        }
        _ if !returns_generator(context, &function.body, &return_hint.hint) => {
            for r#return in mago_syntax::utils::find_returns_in_block(&function.body) {
                if r#return.value.is_none() {
                    context.report(
                        Issue::error(format!("Function `{name}` with a return type must return a value."))
                            .with_annotation(
                                Annotation::primary(r#return.span()).with_message("Empty return statement found here."),
                            )
                            .with_annotation(
                                Annotation::secondary(function.span())
                                    .with_message(format!("Function `{fqfn}` defined here.")),
                            )
                            .with_note("Did you mean `return null;` instead of `return;`?")
                            .with_help("Add a return value to the statement."),
                    );
                }
            }
        }
        _ => {}
    }
}

#[inline]
pub fn check_arrow_function(arrow_function: &ArrowFunction, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::ArrowFunctions) {
        let issue = Issue::error("The `fn` keyword for arrow functions is only available in PHP 7.4 and later.")
            .with_annotation(
                Annotation::primary(arrow_function.span()).with_message("Arrow function uses `fn` keyword."),
            );

        context.report(issue);
    }

    check_for_promoted_properties_outside_constructor(&arrow_function.parameter_list, context);

    // Check for $this usage in static arrow functions
    if arrow_function.r#static.is_some()
        && let Some(this_span) = contains_this_in_expression(arrow_function.expression)
    {
        context.report(
            Issue::error("Cannot use `$this` in a static arrow function.")
                .with_annotation(
                    Annotation::primary(this_span).with_message("`$this` is not available in static context."),
                )
                .with_annotation(
                    Annotation::secondary(arrow_function.r#static.unwrap().span)
                        .with_message("Arrow function is declared as static here."),
                )
                .with_help("Remove the `static` keyword or avoid using `$this` in the arrow function body."),
        );
    }

    if let Some(return_hint) = &arrow_function.return_type_hint {
        // while technically valid, it is not possible to return `void` from an arrow function
        // because the return value is always inferred from the body, even if the body does
        // not return a value, in the case it throws or exits the process.
        //
        // see: https://3v4l.org/VgoiO
        match &return_hint.hint {
            Hint::Void(_) => {
                context.report(
                    Issue::error("Arrow function cannot have a return type of `void`.")
                        .with_annotation(
                            Annotation::primary(return_hint.hint.span())
                                .with_message("Return type `void` is not valid for an arrow function."),
                        )
                        .with_annotation(
                            Annotation::secondary(arrow_function.r#fn.span)
                                .with_message("Arrow function defined here."),
                        )
                        .with_help("Remove the `void` return type hint, or replace it with a valid type."),
                );
            }
            Hint::Never(_) if !context.version.is_supported(Feature::NeverReturnTypeInArrowFunction) => {
                context.report(
                    Issue::error("The `never` return type in arrow functions is only available in PHP 8.2 and later.")
                        .with_annotation(
                            Annotation::primary(return_hint.hint.span())
                                .with_message("Return type `never` is not valid for an arrow function."),
                        )
                        .with_annotation(
                            Annotation::secondary(arrow_function.r#fn.span)
                                .with_message("Arrow function defined here."),
                        ),
                );
            }
            _ => {}
        }
    }
}

#[inline]
pub fn check_closure<'arena>(closure: &Closure<'arena>, context: &mut Context<'_, '_, 'arena>) {
    check_for_promoted_properties_outside_constructor(&closure.parameter_list, context);

    // Check for $this usage in static closures
    if closure.r#static.is_some()
        && let Some(this_span) = contains_this_in_block(&closure.body)
    {
        context.report(
            Issue::error("Cannot use `$this` in a static closure.")
                .with_annotation(
                    Annotation::primary(this_span).with_message("`$this` is not available in static context."),
                )
                .with_annotation(
                    Annotation::secondary(closure.r#static.unwrap().span)
                        .with_message("Closure is declared as static here."),
                )
                .with_help("Remove the `static` keyword or avoid using `$this` in the closure body."),
        );
    }

    if !context.version.is_supported(Feature::TrailingCommaInClosureUseList)
        && let Some(trailing_comma) = &closure.use_clause.as_ref().and_then(|u| u.variables.get_trailing_token())
    {
        context.report(
                Issue::error("Trailing comma in closure use list is only available in PHP 8.0 and later.")
                .with_annotation(
                    Annotation::primary(trailing_comma.span_for(context.source_file.file_id())).with_message("Trailing comma found here."),
                )
                .with_help(
                    "Remove the trailing comma to make the code compatible with PHP 7.4 and earlier versions, or upgrade to PHP 8.0 or later.",
                )
            );
    }

    let hint = if let Some(return_hint) = &closure.return_type_hint {
        &return_hint.hint
    } else {
        return;
    };

    let returns = mago_syntax::utils::find_returns_in_block(&closure.body);

    match &hint {
        Hint::Void(_) => {
            for r#return in returns {
                if let Some(val) = &r#return.value {
                    context.report(
                        Issue::error("Closure with a return type of `void` must not return a value.")
                            .with_annotation(
                                Annotation::primary(val.span())
                                    .with_message("This value is not allowed with a `void` return type."),
                            )
                            .with_annotation(
                                Annotation::secondary(closure.span()).with_message("Closure defined here."),
                            )
                            .with_help(
                                "Remove the return value, or change the return type hint to an appropriate type.",
                            ),
                    );
                }
            }
        }
        Hint::Never(_) => {
            for r#return in returns {
                context.report(
                    Issue::error("Closure with a return type of `never` must not include a return statement.")
                        .with_annotation(
                            Annotation::primary(r#return.span())
                                .with_message("Return statement is not allowed with a `never` return type."),
                        )
                        .with_annotation(Annotation::secondary(closure.span()).with_message("Closure defined here."))
                        .with_help("Remove the return statement, or change the return type hint to a compatible type."),
                );
            }
        }
        _ if !returns_generator(context, &closure.body, hint) => {
            for r#return in returns {
                if r#return.value.is_none() {
                    context.report(
                        Issue::error("Closure with a return type must return a value.")
                            .with_annotation(Annotation::primary(r#return.span()).with_message("Missing return value."))
                            .with_annotation(
                                Annotation::secondary(closure.span()).with_message("Closure defined here."),
                            )
                            .with_note("Did you mean `return null;` instead of `return;`?")
                            .with_help("Add a return value that matches the expected return type."),
                    );
                }
            }
        }
        _ => {}
    }
}

#[inline]
pub fn check_return_type_hint(
    function_like_return_type_hint: &FunctionLikeReturnTypeHint,
    context: &mut Context<'_, '_, '_>,
) {
    match &function_like_return_type_hint.hint {
        Hint::Union(union_hint) if !context.version.is_supported(Feature::NativeUnionTypes) => {
            context.report(
                Issue::error(
                    "Union type hints (e.g. `int|float`) are only available in PHP 8.0 and above."
                )
                .with_annotation(Annotation::primary(union_hint.span()).with_message("Union type hint used here."))
                .with_note(
                    "Union type hints are only available in PHP 8.0 and above. Consider using a different approach.",
                )
                .with_help("Remove the union type hint to make the code compatible with PHP 7.4 and earlier versions, or upgrade to PHP 8.0 or later."),
                        );
        }
        Hint::Static(r#static) if !context.version.is_supported(Feature::StaticReturnTypeHint) => {
            context.report(
                Issue::error("Static return type hints are only available in PHP 8.0 and above.").with_annotation(
                    Annotation::primary(r#static.span()).with_message("Static return type hint used here."),
                )
                .with_help("Remove the static return type hint to make the code compatible with PHP 7.4 and earlier versions, or upgrade to PHP 8.0 or later."),
            );
        }
        _ => {}
    }
}

#[inline]
pub fn check_for_promoted_properties_outside_constructor(
    parameter_list: &FunctionLikeParameterList,
    context: &mut Context<'_, '_, '_>,
) {
    for parameter in &parameter_list.parameters {
        if parameter.is_promoted_property() {
            context.report(
                Issue::error("Promoted properties are not allowed outside of constructors.")
                    .with_annotation(
                        Annotation::primary(parameter.span()).with_message("Promoted property found here."),
                    )
                    .with_help("Move this promoted property to the constructor, or remove the promotion."),
            );
        }
    }
}
