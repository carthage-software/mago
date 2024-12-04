use fennec_ast::*;

use crate::document::Document;
use crate::document::Group;
use crate::format::Format;
use crate::Formatter;

use super::call_arguments::print_call_arguments;
use super::call_node::CallLikeNode;
use super::Line;

pub(super) struct MethodChain<'a> {
    pub base: &'a Expression,
    pub calls: Vec<CallLikeNode<'a>>,
}

pub(super) fn collect_method_call_chain<'a>(expr: &'a Expression) -> Option<MethodChain<'a>> {
    let mut calls = Vec::new();
    let mut current_expr = expr;

    loop {
        match current_expr {
            Expression::Call(call) => {
                calls.push(CallLikeNode::Call(call));

                match call {
                    Call::Method(method_call) => {
                        current_expr = method_call.object.as_ref();
                    }
                    Call::NullSafeMethod(null_safe_method_call) => {
                        current_expr = null_safe_method_call.object.as_ref();
                    }
                    Call::StaticMethod(static_method_call) => {
                        current_expr = static_method_call.class.as_ref();
                    }
                    _ => {
                        break;
                    }
                }
            }
            _ => {
                break;
            }
        }
    }

    if calls.is_empty() {
        None
    } else {
        calls.reverse();

        Some(MethodChain { base: current_expr, calls })
    }
}

pub(super) fn print_method_call_chain<'a>(method_chain: &MethodChain<'a>, f: &mut Formatter<'a>) -> Document<'a> {
    let mut parts = Vec::new();

    let mut calls_iter = method_chain.calls.iter();

    parts.push(method_chain.base.format(f));

    // Handle the first method call
    if !f.settings.method_chain_breaking_style.is_next_line() {
        if let Some(first_chain_link) = calls_iter.next() {
            // Format the base object and first method call together
            let (operator, method) = match first_chain_link {
                CallLikeNode::Call(Call::Method(c)) => (Document::String("->"), c.method.format(f)),
                CallLikeNode::Call(Call::NullSafeMethod(c)) => (Document::String("?->"), c.method.format(f)),
                CallLikeNode::Call(Call::StaticMethod(c)) => (Document::String("::"), c.method.format(f)),
                _ => unreachable!(),
            };

            parts.push(operator);
            parts.push(method);
            parts.push(print_call_arguments(f, first_chain_link));
        }
    }

    // Now handle the remaining method calls
    for chain_link in calls_iter {
        let mut contents = vec![Document::Line(Line::softline())];
        contents.extend(match chain_link {
            CallLikeNode::Call(Call::Method(c)) => vec![Document::String("->"), c.method.format(f)],
            CallLikeNode::Call(Call::NullSafeMethod(c)) => vec![Document::String("?->"), c.method.format(f)],
            CallLikeNode::Call(Call::StaticMethod(c)) => vec![Document::String("::"), c.method.format(f)],
            _ => unreachable!(),
        });

        contents.push(print_call_arguments(f, chain_link));

        parts.push(Document::Indent(contents));
    }

    // Wrap everything in a group to manage line breaking
    Document::Group(Group::new(parts))
}
