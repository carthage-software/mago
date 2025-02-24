use mago_ast::*;

use crate::Formatter;
use crate::document::Align;
use crate::document::Document;
use crate::document::IndentIfBreak;
use crate::document::Separator;

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

pub fn is_non_empty_array_like_expression(mut expression: &Expression) -> bool {
    while let Expression::Parenthesized(parenthesized) = expression {
        expression = &parenthesized.expression;
    }

    match expression {
        Expression::Array(Array { elements, .. })
        | Expression::List(List { elements, .. })
        | Expression::LegacyArray(LegacyArray { elements, .. }) => !elements.is_empty(),
        _ => false,
    }
}

pub fn is_at_call_like_expression(f: &Formatter<'_>) -> bool {
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

pub fn is_at_callee(f: &Formatter<'_>) -> bool {
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

pub fn will_break(document: &mut Document<'_>) -> bool {
    let check_array = |array: &mut Vec<Document<'_>>| array.iter_mut().rev().any(|doc| will_break(doc));

    match document {
        Document::BreakParent => true,
        Document::Group(group) => {
            if group.should_break {
                return true;
            }
            if let Some(expanded_states) = &mut group.expanded_states {
                if expanded_states.iter_mut().rev().any(will_break) {
                    return true;
                }
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
        Document::String(_) | Document::LineSuffixBoundary | Document::Trim(_) => false,
    }
}

pub fn replace_end_of_line(document: Document<'_>, replacement: Separator) -> Document<'_> {
    let Document::String(text) = document else {
        return document;
    };

    Document::Array(Document::join(text.split("\n").map(Document::String).collect(), replacement))
}
