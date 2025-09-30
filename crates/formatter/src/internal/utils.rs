use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;
use unicode_width::UnicodeWidthStr;

use mago_syntax::ast::*;

use crate::document::Align;
use crate::document::Document;
use crate::document::IndentIfBreak;
use crate::document::Separator;
use crate::internal::FormatterState;
use crate::internal::format::call_arguments::should_break_all_arguments;
use crate::internal::format::misc::is_breaking_expression;
use crate::internal::format::misc::is_simple_single_line_expression;

use super::format::call_arguments::should_expand_first_arg;
use super::format::call_arguments::should_expand_last_arg;

#[inline]
pub const fn has_naked_left_side(expression: &Expression) -> bool {
    matches!(
        expression,
        Expression::Binary(_)
            | Expression::UnaryPostfix(_)
            | Expression::Assignment(_)
            | Expression::Conditional(_)
            | Expression::ArrayAccess(_)
            | Expression::ArrayAppend(_)
            | Expression::Call(_)
            | Expression::Access(_)
            | Expression::ClosureCreation(_)
    )
}

#[inline]
pub fn get_left_side<'arena>(expression: &'arena Expression<'arena>) -> Option<&'arena Expression<'arena>> {
    match expression {
        Expression::Binary(binary) => Some(binary.lhs),
        Expression::UnaryPostfix(unary) => Some(unary.operand),
        Expression::Assignment(assignment) => Some(assignment.lhs),
        Expression::Conditional(conditional) => Some(conditional.condition),
        Expression::ArrayAccess(array_access) => Some(array_access.array),
        Expression::ArrayAppend(array_append) => Some(array_append.array),
        Expression::Call(call) => Some(match call {
            Call::Function(function_call) => function_call.function,
            Call::Method(method_call) => method_call.object,
            Call::NullSafeMethod(null_safe_method_call) => null_safe_method_call.object,
            Call::StaticMethod(static_method_call) => static_method_call.class,
        }),
        Expression::Access(access) => Some(match access {
            Access::Property(property_access) => property_access.object,
            Access::NullSafeProperty(null_safe_property_access) => null_safe_property_access.object,
            Access::StaticProperty(static_property_access) => static_property_access.class,
            Access::ClassConstant(class_constant_access) => class_constant_access.class,
        }),
        Expression::ClosureCreation(closure_creation) => Some(match closure_creation {
            ClosureCreation::Function(function_closure_creation) => function_closure_creation.function,
            ClosureCreation::Method(method_closure_creation) => method_closure_creation.object,
            ClosureCreation::StaticMethod(static_method_closure_creation) => static_method_closure_creation.class,
        }),
        _ => None,
    }
}

#[inline]
pub fn is_at_call_like_expression(f: &FormatterState<'_, '_>) -> bool {
    let Some(grant_parent) = f.grandparent_node() else {
        return false;
    };

    matches!(
        grant_parent,
        Node::FunctionCall(_)
            | Node::MethodCall(_)
            | Node::StaticMethodCall(_)
            | Node::NullSafeMethodCall(_)
            | Node::FunctionClosureCreation(_)
            | Node::MethodClosureCreation(_)
            | Node::StaticMethodClosureCreation(_)
    )
}

#[inline]
pub fn unwrap_parenthesized<'ast, 'arena>(mut expression: &'ast Expression<'arena>) -> &'ast Expression<'arena> {
    while let Expression::Parenthesized(parenthesized) = expression {
        expression = parenthesized.expression;
    }

    expression
}

#[inline]
pub fn is_at_callee(f: &FormatterState<'_, '_>) -> bool {
    let Node::Expression(expression) = f.parent_node() else {
        return false;
    };

    let Some(parent) = f.grandparent_node() else {
        return false;
    };

    match parent {
        Node::FunctionCall(call) => call.function == expression,
        Node::MethodCall(call) => call.object == expression,
        Node::StaticMethodCall(call) => call.class == expression,
        Node::NullSafeMethodCall(call) => call.object == expression,
        Node::FunctionClosureCreation(closure) => closure.function == expression,
        Node::MethodClosureCreation(closure) => closure.object == expression,
        Node::StaticMethodClosureCreation(closure) => closure.class == expression,
        _ => false,
    }
}

#[inline]
pub fn will_break<'arena>(document: &'arena Document<'arena>) -> bool {
    let check_array = |array: &Vec<'arena, Document<'arena>>| array.iter().rev().any(|doc| will_break(doc));

    match document {
        Document::BreakParent => true,
        Document::Line(doc) => doc.hard,
        Document::Group(group) => {
            if *group.should_break.borrow() {
                return true;
            }

            check_array(&group.contents)
        }
        Document::IfBreak(d) => will_break(d.flat_content) || will_break(d.break_contents),
        Document::Array(contents)
        | Document::Indent(contents)
        | Document::LineSuffix(contents)
        | Document::IndentIfBreak(IndentIfBreak { contents, .. })
        | Document::Align(Align { contents, .. }) => check_array(contents),
        Document::Fill(doc) => check_array(&doc.parts),
        _ => false,
    }
}

#[inline]
pub fn replace_end_of_line<'arena>(
    f: &FormatterState<'_, 'arena>,
    document: Document<'arena>,
    replacement: Separator,
    halted_compilation: bool,
) -> Document<'arena> {
    let Document::String(text) = document else {
        return document;
    };

    // Do not modify the content if the compilation was halted.
    if halted_compilation {
        return Document::String(text);
    }

    Document::Array(Document::join(
        f.arena,
        text.split("\n").map(Document::String).collect_in::<Vec<_>>(f.arena),
        replacement,
    ))
}

#[inline]
pub fn could_expand_value<'arena>(
    f: &FormatterState<'_, 'arena>,
    value: &'arena Expression<'arena>,
    nested_args: bool,
) -> bool {
    match value {
        Expression::Array(expr) => {
            // If empty, we can't expand it.
            if expr.elements.is_empty() {
                return false;
            }

            // If we have more than 3 elements, we should expand it.
            // This is a somewhat arbitrary limit, but it helps to keep things readable.
            if expr.elements.len() >= 4 {
                return true;
            }

            // If it has at least one key-value pair, we should expand it.
            if expr.elements.iter().any(|element| element.is_key_value()) {
                return true;
            };

            expr.elements.iter().any(|element| match element {
                ArrayElement::Variadic(e) => !is_simple_single_line_expression(f, e.value),
                ArrayElement::Value(e) => !is_simple_single_line_expression(f, e.value),
                ArrayElement::KeyValue(key_value) => could_expand_value(f, key_value.value, nested_args),
                _ => false,
            })
        }
        Expression::LegacyArray(expr) => !expr.elements.is_empty(),
        Expression::List(expr) => !expr.elements.is_empty(),
        Expression::AnonymousClass(_) => true,
        Expression::Closure(_) => true,
        Expression::Match(m) => !m.arms.is_empty(),
        Expression::Binary(operation) => could_expand_value(f, operation.lhs, nested_args),
        Expression::ArrowFunction(arrow_function) => match arrow_function.expression {
            Expression::Call(_) | Expression::Conditional(_) => true,
            other => is_breaking_expression(f, other, true),
        },
        Expression::Instantiation(instantiation) => {
            let Expression::Identifier(_) = instantiation.class else {
                return false;
            };

            let Some(arguments) = instantiation.argument_list.as_ref() else {
                return false;
            };

            let arguments_len = arguments.arguments.len();

            let Some(first_arg) = arguments.arguments.first() else {
                return false;
            };

            if arguments_len == 1 {
                return could_expand_value(f, first_arg.value(), true);
            }

            should_break_all_arguments(f, arguments, false)
                || should_expand_first_arg(f, arguments, true)
                || should_expand_last_arg(f, arguments, true)
        }
        Expression::Literal(Literal::String(literal_string)) => {
            literal_string.raw.contains('\n') || literal_string.raw.contains('\r')
        }
        Expression::CompositeString(composite_string) => composite_string.parts().iter().any(|part| match part {
            StringPart::Literal(literal_string) => {
                literal_string.value.contains('\n') || literal_string.value.contains('\r')
            }
            _ => false,
        }),
        Expression::Call(call) if !nested_args => {
            let argument_list = call.get_argument_list();

            if let Some(first_arg) = argument_list.arguments.first()
                && argument_list.arguments.len() == 1
            {
                return could_expand_value(f, first_arg.value(), true);
            }

            should_break_all_arguments(f, argument_list, false)
                || should_expand_first_arg(f, argument_list, true)
                || should_expand_last_arg(f, argument_list, true)
        }
        _ => false,
    }
}

#[inline]
pub fn string_width(s: &str) -> usize {
    let line = s.lines().last().unwrap_or("");

    if line.contains("الله") {
        // The word "الله" is a special case, as it is usually rendered as a single glyph
        // while being 4 characters wide. This is a hack to handle this case.
        line.replace("الله", "_").width()
    } else {
        line.width()
    }
}
