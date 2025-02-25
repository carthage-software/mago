use mago_ast::*;

use crate::Formatter;
use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::format::Format;
use crate::format::call_arguments::print_call_arguments;
use crate::format::call_node::CallLikeNode;
use crate::parens::instantiation_needs_parens;

pub(super) struct MethodChain<'a> {
    pub base: &'a Expression,
    pub calls: Vec<CallLikeNode<'a>>,
}

pub(super) fn collect_method_call_chain(expr: &Expression) -> Option<MethodChain<'_>> {
    let mut calls = Vec::new();
    let mut current_expr = expr;

    while let Expression::Call(call) = current_expr {
        current_expr = match call {
            Call::Method(method_call) => {
                calls.push(CallLikeNode::Call(call));

                method_call.object.as_ref()
            }
            Call::NullSafeMethod(null_safe_method_call) => {
                calls.push(CallLikeNode::Call(call));

                null_safe_method_call.object.as_ref()
            }
            _ => {
                break;
            }
        };
    }

    if calls.is_empty() {
        None
    } else {
        calls.reverse();

        Some(MethodChain { base: current_expr, calls })
    }
}

pub(super) fn print_method_call_chain<'a>(method_chain: &MethodChain<'a>, f: &mut Formatter<'a>) -> Document<'a> {
    let base_document = method_chain.base.format(f);
    let mut parts = if base_needs_parerns(f, method_chain.base) {
        vec![Document::String("("), base_document, Document::String(")")]
    } else {
        vec![base_document]
    };

    let mut calls_iter = method_chain.calls.iter();

    // Handle the first method call
    if !f.settings.method_chain_breaking_style.is_next_line() {
        if let Some(first_chain_link) = calls_iter.next() {
            // Format the base object and first method call together
            let (operator, method) = match first_chain_link {
                CallLikeNode::Call(Call::Method(c)) => (Document::String("->"), c.method.format(f)),
                CallLikeNode::Call(Call::NullSafeMethod(c)) => (Document::String("?->"), c.method.format(f)),
                _ => unreachable!(),
            };

            parts.push(operator);
            parts.push(method);

            parts.push(Document::Group(Group::new(vec![print_call_arguments(f, first_chain_link)])));
        }
    }

    // Now handle the remaining method calls
    for chain_link in calls_iter {
        let mut contents = vec![Document::Line(Line::hard())];
        contents.extend(match chain_link {
            CallLikeNode::Call(Call::Method(c)) => vec![Document::String("->"), c.method.format(f)],
            CallLikeNode::Call(Call::NullSafeMethod(c)) => vec![Document::String("?->"), c.method.format(f)],
            _ => unreachable!(),
        });

        contents.push(Document::Group(Group::new(vec![print_call_arguments(f, chain_link)])));

        parts.push(Document::Indent(contents));
    }

    parts.push(Document::BreakParent);

    // Wrap everything in a group to manage line breaking
    Document::Group(Group::new(parts))
}

fn base_needs_parerns(f: &Formatter<'_>, base: &Expression) -> bool {
    if let Expression::Parenthesized(parenthesized) = base {
        return base_needs_parerns(f, &parenthesized.expression);
    }

    match base {
        Expression::Instantiation(instantiation) => instantiation_needs_parens(f, instantiation),
        Expression::Binary(_)
        | Expression::UnaryPrefix(_)
        | Expression::UnaryPostfix(_)
        | Expression::Assignment(_)
        | Expression::Conditional(_)
        | Expression::AnonymousClass(_)
        | Expression::Closure(_)
        | Expression::ArrowFunction(_)
        | Expression::Match(_)
        | Expression::Yield(_)
        | Expression::Clone(_) => true,
        _ => false,
    }
}
