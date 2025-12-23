use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;
use mago_syntax::ast::Argument;
use mago_syntax::ast::ArgumentList;
use mago_syntax::ast::ClassConstantAccess;
use mago_syntax::ast::ClassLikeConstantSelector;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::ConstantAccess;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::StaticMethodCall;
use mago_syntax::ast::Variable;
use unicode_width::UnicodeWidthStr;

use mago_syntax::ast::Access;
use mago_syntax::ast::ArrayElement;
use mago_syntax::ast::Call;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Node;
use mago_syntax::ast::PartialApplication;
use mago_syntax::ast::StringPart;

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
            | Expression::PartialApplication(_)
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
        Expression::PartialApplication(partial_application) => Some(match partial_application {
            PartialApplication::Function(function_partial_application) => function_partial_application.function,
            PartialApplication::Method(method_partial_application) => method_partial_application.object,
            PartialApplication::StaticMethod(static_method_partial_application) => {
                static_method_partial_application.class
            }
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
            | Node::FunctionPartialApplication(_)
            | Node::MethodPartialApplication(_)
            | Node::StaticMethodPartialApplication(_)
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
        Node::FunctionPartialApplication(partial_application) => partial_application.function == expression,
        Node::MethodPartialApplication(partial_application) => partial_application.object == expression,
        Node::StaticMethodPartialApplication(partial_application) => partial_application.class == expression,
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
        text.split('\n').map(Document::String).collect_in::<Vec<_>>(f.arena),
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
            if expr.elements.iter().any(ArrayElement::is_key_value) {
                return true;
            }

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
        Expression::ArrowFunction(arrow_function) => match unwrap_parenthesized(arrow_function.expression) {
            Expression::Call(_) => true,
            Expression::Conditional(c) => c.then.is_some(),
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

pub fn foreach_binary_operand<F>(expr: &Expression<'_>, f: &mut F)
where
    F: FnMut(&Expression<'_>),
{
    let Expression::Binary(binary) = expr else {
        return;
    };

    f(binary.rhs);
    foreach_binary_operand(binary.rhs, f);

    f(binary.lhs);
    foreach_binary_operand(binary.lhs, f);
}

pub fn get_expression_width(element: &Expression<'_>) -> Option<usize> {
    fn get_argument_width(argument: &Argument<'_>) -> Option<usize> {
        match argument {
            Argument::Positional(arg) => match arg.ellipsis {
                Some(_) => get_expression_width(&arg.value).map(|width| width + 3),
                None => get_expression_width(&arg.value),
            },
            Argument::Named(arg) => get_expression_width(&arg.value).map(|mut width| {
                width += 2;
                width += arg.name.value.width();
                width
            }),
        }
    }

    fn get_argument_list_width(argument_list: &ArgumentList<'_>) -> Option<usize> {
        let mut width = 2;
        for (i, argument) in argument_list.arguments.iter().enumerate() {
            if i > 0 {
                width += 2;
            }

            width += get_argument_width(argument)?;
        }

        Some(width)
    }

    Some(match element {
        Expression::Literal(literal) => match literal {
            Literal::String(literal_string) => string_width(literal_string.raw),
            Literal::Integer(literal_integer) => literal_integer.raw.width(),
            Literal::Float(literal_float) => literal_float.raw.width(),
            Literal::True(_) => 4,
            Literal::False(_) => 5,
            Literal::Null(_) => 4,
        },
        Expression::MagicConstant(magic_constant) => string_width(magic_constant.value().value),
        Expression::ConstantAccess(ConstantAccess { name: Identifier::Local(local) })
        | Expression::Identifier(Identifier::Local(local)) => string_width(local.value),
        Expression::Variable(Variable::Direct(variable)) => string_width(variable.name),
        Expression::Call(Call::Function(FunctionCall { function, argument_list })) => {
            let function_width = get_expression_width(function)?;
            let args_width = get_argument_list_width(argument_list)?;

            function_width + args_width
        }
        Expression::Call(Call::StaticMethod(StaticMethodCall {
            class,
            method: ClassLikeMemberSelector::Identifier(method),
            argument_list,
            ..
        })) => {
            let class_width = get_expression_width(class)?;
            let method_width = string_width(method.value);
            let args_width = get_argument_list_width(argument_list)?;

            class_width + 2 + method_width + args_width
        }
        Expression::Access(Access::ClassConstant(ClassConstantAccess {
            class,
            constant: ClassLikeConstantSelector::Identifier(constant),
            ..
        })) => {
            return get_expression_width(class).map(|class| class + 2 + string_width(constant.value));
        }
        _ => {
            return None;
        }
    })
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
