use mago_span::*;
use mago_syntax::ast::*;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::format::call_arguments::print_call_arguments;

use super::member_access::format_access_operator;
use super::misc;

pub(super) enum CallLikeNode<'a> {
    Call(&'a Call),
    Instantiation(&'a Instantiation),
    Attribute(&'a Attribute),
    DieConstruct(&'a DieConstruct),
    ExitConstruct(&'a ExitConstruct),
}

impl<'a> CallLikeNode<'a> {
    #[inline]
    pub const fn is_instantiation(&self) -> bool {
        matches!(self, CallLikeNode::Instantiation(_))
    }

    #[inline]
    pub const fn is_exit_or_die_construct(&self) -> bool {
        matches!(self, CallLikeNode::DieConstruct(_) | CallLikeNode::ExitConstruct(_))
    }

    #[inline]
    pub const fn is_attribute(&self) -> bool {
        matches!(self, CallLikeNode::Attribute(_))
    }

    pub fn arguments(&self) -> Option<&'a ArgumentList> {
        match self {
            CallLikeNode::Call(call) => Some(match call {
                Call::Function(c) => &c.argument_list,
                Call::Method(c) => &c.argument_list,
                Call::NullSafeMethod(c) => &c.argument_list,
                Call::StaticMethod(c) => &c.argument_list,
            }),
            CallLikeNode::Instantiation(new) => new.arguments.as_ref(),
            CallLikeNode::Attribute(attr) => attr.arguments.as_ref(),
            CallLikeNode::DieConstruct(die) => die.arguments.as_ref(),
            CallLikeNode::ExitConstruct(exit) => exit.arguments.as_ref(),
        }
    }
}

impl HasSpan for CallLikeNode<'_> {
    fn span(&self) -> Span {
        match self {
            CallLikeNode::Call(call) => call.span(),
            CallLikeNode::Instantiation(new) => new.span(),
            CallLikeNode::Attribute(attr) => attr.span(),
            CallLikeNode::DieConstruct(die) => die.span(),
            CallLikeNode::ExitConstruct(exit) => exit.span(),
        }
    }
}

pub(super) fn print_call_like_node<'a>(f: &mut FormatterState<'a>, node: CallLikeNode<'a>) -> Document<'a> {
    // format the callee-like expression
    let mut parts = match node {
        CallLikeNode::Call(c) => match c {
            Call::Function(c) => vec![c.function.format(f)],
            Call::StaticMethod(c) => vec![c.class.format(f), Document::String("::"), c.method.format(f)],
            _ => {
                return print_access_call_node(f, c);
            }
        },
        CallLikeNode::Instantiation(i) => vec![i.new.format(f), Document::space(), i.class.format(f)],
        CallLikeNode::Attribute(a) => vec![a.name.format(f)],
        CallLikeNode::DieConstruct(d) => vec![d.die.format(f)],
        CallLikeNode::ExitConstruct(e) => vec![e.exit.format(f)],
    };

    parts.push(print_call_arguments(f, node));

    Document::Group(Group::new(parts))
}

fn print_access_call_node<'a>(f: &mut FormatterState<'a>, node: &'a Call) -> Document<'a> {
    let (base, operator, operator_str, selector) = match node {
        Call::Method(method_call) => (&method_call.object, method_call.arrow, "->", &method_call.method),
        Call::NullSafeMethod(null_safe_method_call) => (
            &null_safe_method_call.object,
            null_safe_method_call.question_mark_arrow,
            "?->",
            &null_safe_method_call.method,
        ),
        _ => unreachable!(),
    };

    let should_break = f.has_inner_comment(Span::new(base.span().end, operator.start))
        || (f.settings.preserve_breaking_member_access_chain
            && misc::has_new_line_in_range(f.source_text, base.span().end.offset, operator.start.offset));

    if should_break {
        Document::Group(Group::new(vec![
            base.format(f),
            Document::Line(Line::hard()),
            format_access_operator(f, operator, operator_str),
            selector.format(f),
            print_call_arguments(f, CallLikeNode::Call(node)),
        ]))
    } else {
        Document::Group(Group::new(vec![
            base.format(f),
            format_access_operator(f, operator, operator_str),
            selector.format(f),
            print_call_arguments(f, CallLikeNode::Call(node)),
        ]))
    }
}
