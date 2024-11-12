use fennec_ast::*;

use crate::document::Document;
use crate::document::Group;
use crate::group;
use crate::indent;
use crate::static_str;
use crate::token;
use crate::Formatter;

use super::Format;
use super::Line;

pub(super) struct MethodChain<'a> {
    pub base: &'a Expression,
    pub calls: Vec<ChainLink<'a>>,
}

pub enum ChainLink<'a> {
    MethodCall(&'a MethodCall),
    NullSafeMethodCall(&'a NullSafeMethodCall),
    StaticMethodCall(&'a StaticMethodCall),
}

pub(super) fn collect_method_call_chain<'a>(expr: &'a Expression) -> Option<MethodChain<'a>> {
    let mut calls = Vec::new();
    let mut current_expr = expr;

    loop {
        match current_expr {
            Expression::Call(call) => match call {
                Call::Method(method_call) => {
                    calls.push(ChainLink::MethodCall(method_call));
                    current_expr = method_call.object.as_ref();
                }
                Call::NullSafeMethod(null_safe_method_call) => {
                    calls.push(ChainLink::NullSafeMethodCall(null_safe_method_call));
                    current_expr = null_safe_method_call.object.as_ref();
                }
                Call::StaticMethod(static_method_call) => {
                    calls.push(ChainLink::StaticMethodCall(static_method_call));
                    current_expr = static_method_call.class.as_ref();
                }
                _ => {
                    break;
                }
            },
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
    if let Some(first_chain_link) = calls_iter.next() {
        if !f.settings.method_chain_breaking_style.is_next_line() {
            // Format the base object and first method call together
            let first_call = match first_chain_link {
                ChainLink::MethodCall(method_call) => {
                    group!(static_str!("->"), method_call.method.format(f), method_call.arguments.format(f))
                }
                ChainLink::NullSafeMethodCall(null_safe_method_call) => group!(
                    static_str!("?->"),
                    null_safe_method_call.method.format(f),
                    null_safe_method_call.arguments.format(f)
                ),
                ChainLink::StaticMethodCall(static_method_call) => {
                    group!(
                        static_str!("::"),
                        static_method_call.method.format(f),
                        static_method_call.arguments.format(f)
                    )
                }
            };

            parts.push(first_call);
        }

        // Now handle the remaining method calls
        for chain_link in calls_iter {
            let (operator_doc, method_doc, args_doc) = match chain_link {
                ChainLink::MethodCall(method_call) => {
                    (token!(f, method_call.arrow, "->"), method_call.method.format(f), method_call.arguments.format(f))
                }
                ChainLink::NullSafeMethodCall(null_safe_method_call) => (
                    token!(f, null_safe_method_call.question_mark_arrow, "?->"),
                    null_safe_method_call.method.format(f),
                    null_safe_method_call.arguments.format(f),
                ),
                ChainLink::StaticMethodCall(static_method_call) => (
                    token!(f, static_method_call.double_colon, "::"),
                    static_method_call.method.format(f),
                    static_method_call.arguments.format(f),
                ),
            };

            // Combine operator and method call into a group
            let mut call_parts = vec![];
            call_parts.push(Document::Line(Line::hardline()));
            call_parts.push(operator_doc);
            call_parts.push(method_doc);
            call_parts.push(args_doc);

            // Add indent the method call
            parts.push(indent!(group!(@call_parts)));
        }
    } else {
        // No method calls, just the base object, should not happen, but just in case.
        parts.push(method_chain.base.format(f));
    }

    // Wrap everything in a group to manage line breaking
    Document::Group(Group::new(parts))
}
