use unicode_width::UnicodeWidthStr;

use mago_syntax::ast::*;

use crate::document::Align;
use crate::document::Document;
use crate::document::IndentIfBreak;
use crate::document::Separator;
use crate::internal::FormatterState;

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
pub const fn get_left_side(expression: &Expression) -> Option<&Expression> {
    match expression {
        Expression::Binary(binary) => Some(&binary.lhs),
        Expression::UnaryPostfix(unary) => Some(&unary.operand),
        Expression::Assignment(assignment) => Some(&assignment.lhs),
        Expression::Conditional(conditional) => Some(&conditional.condition),
        Expression::ArrayAccess(array_access) => Some(&array_access.array),
        Expression::ArrayAppend(array_append) => Some(&array_append.array),
        Expression::Call(call) => Some(match call {
            Call::Function(function_call) => &function_call.function,
            Call::Method(method_call) => &method_call.object,
            Call::NullSafeMethod(null_safe_method_call) => &null_safe_method_call.object,
            Call::StaticMethod(static_method_call) => &static_method_call.class,
        }),
        Expression::Access(access) => Some(match access {
            Access::Property(property_access) => &property_access.object,
            Access::NullSafeProperty(null_safe_property_access) => &null_safe_property_access.object,
            Access::StaticProperty(static_property_access) => &static_property_access.class,
            Access::ClassConstant(class_constant_access) => &class_constant_access.class,
        }),
        Expression::ClosureCreation(closure_creation) => Some(match closure_creation {
            ClosureCreation::Function(function_closure_creation) => &function_closure_creation.function,
            ClosureCreation::Method(method_closure_creation) => &method_closure_creation.object,
            ClosureCreation::StaticMethod(static_method_closure_creation) => &static_method_closure_creation.class,
        }),
        _ => None,
    }
}

#[inline]
pub fn is_at_call_like_expression(f: &FormatterState<'_>) -> bool {
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
pub const fn unwrap_parenthesized(mut expression: &Expression) -> &Expression {
    while let Expression::Parenthesized(parenthesized) = expression {
        expression = &parenthesized.expression;
    }

    expression
}

#[inline]
pub fn is_at_callee(f: &FormatterState<'_>) -> bool {
    let Node::Expression(expression) = f.parent_node() else {
        return false;
    };

    let Some(parent) = f.grandparent_node() else {
        return false;
    };

    match parent {
        Node::FunctionCall(call) => call.function.as_ref() == expression,
        Node::MethodCall(call) => call.object.as_ref() == expression,
        Node::StaticMethodCall(call) => call.class.as_ref() == expression,
        Node::NullSafeMethodCall(call) => call.object.as_ref() == expression,
        Node::FunctionClosureCreation(closure) => closure.function.as_ref() == expression,
        Node::MethodClosureCreation(closure) => closure.object.as_ref() == expression,
        Node::StaticMethodClosureCreation(closure) => closure.class.as_ref() == expression,
        _ => false,
    }
}

#[inline]
pub fn will_break(document: &mut Document<'_>) -> bool {
    let check_array = |array: &mut Vec<Document<'_>>| array.iter_mut().rev().any(|doc| will_break(doc));

    match document {
        Document::BreakParent => true,
        Document::Group(group) => {
            if group.should_break {
                return true;
            }
            if let Some(expanded_states) = &mut group.expanded_states
                && expanded_states.iter_mut().rev().any(will_break)
            {
                return true;
            }
            check_array(&mut group.contents)
        }
        Document::IfBreak(d) => will_break(&mut d.break_contents),
        Document::Array(contents)
        | Document::Indent(contents)
        | Document::LineSuffix(contents)
        | Document::IndentIfBreak(IndentIfBreak { contents, .. })
        | Document::Align(Align { contents, .. }) => check_array(contents),
        Document::Fill(doc) => check_array(&mut doc.parts),
        Document::Line(doc) => doc.hard,
        Document::String(_) | Document::LineSuffixBoundary | Document::Trim(_) | Document::DoNotTrim => false,
    }
}

#[inline]
pub fn replace_end_of_line(document: Document<'_>, replacement: Separator, halted_compilation: bool) -> Document<'_> {
    let Document::String(text) = document else {
        return document;
    };

    // Do not modify the content if the compilation was halted.
    if halted_compilation {
        return Document::String(text);
    }

    Document::Array(Document::join(text.split("\n").map(Document::String).collect(), replacement))
}

#[inline]
pub fn could_expand_value(
    f: &FormatterState<'_>,
    value: &Expression,
    arrow_chain_recursion: bool,
    nested_args: bool,
) -> bool {
    match value {
        Expression::Array(expr) => !expr.elements.is_empty(),
        Expression::LegacyArray(expr) => !expr.elements.is_empty(),
        Expression::List(expr) => !expr.elements.is_empty(),
        Expression::AnonymousClass(_) => true,
        Expression::Closure(_) => true,
        Expression::Match(m) => !m.arms.is_empty(),
        Expression::Binary(operation) => could_expand_value(f, &operation.lhs, arrow_chain_recursion, nested_args),
        Expression::ArrowFunction(arrow_function) if !arrow_chain_recursion => match arrow_function.expression.as_ref()
        {
            Expression::Array(_) | Expression::List(_) | Expression::AnonymousClass(_) | Expression::LegacyArray(_) => {
                could_expand_value(f, &arrow_function.expression, true, nested_args)
            }
            Expression::Call(_) | Expression::Conditional(_) => true,
            _ => false,
        },
        Expression::Instantiation(instantiation) => {
            let Expression::Identifier(_) = instantiation.class.as_ref() else {
                return false;
            };

            let Some(arguments) = instantiation.argument_list.as_ref() else {
                return false;
            };

            arguments.arguments.len() > 2
        }
        Expression::Literal(Literal::String(literal_string)) => {
            let string = f.interner.lookup(&literal_string.raw);

            string.contains('\n') || string.contains('\r')
        }
        Expression::CompositeString(composite_string) => composite_string.parts().iter().any(|part| match part {
            StringPart::Literal(literal_string) => {
                let string = f.interner.lookup(&literal_string.value);

                string.contains('\n') || string.contains('\r')
            }
            _ => false,
        }),
        Expression::Call(call) if !nested_args => {
            let argument_list = call.get_argument_list();

            should_expand_first_arg(f, argument_list, true) || should_expand_last_arg(f, argument_list, true)
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
