use mago_ast::*;
use mago_span::*;

use crate::document::Document;
use crate::document::Group;
use crate::format::call_arguments::print_call_arguments;
use crate::format::Format;
use crate::Formatter;

pub(super) enum CallLikeNode<'a> {
    Call(&'a Call),
    Instantiation(&'a Instantiation),
    Attribute(&'a Attribute),
    DieConstruct(&'a DieConstruct),
    ExitConstruct(&'a ExitConstruct),
}

impl<'a> CallLikeNode<'a> {
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

pub(super) fn print_call_like_node<'a>(f: &mut Formatter<'a>, node: CallLikeNode<'a>) -> Document<'a> {
    let mut parts = vec![];

    // format the callee-like expression
    parts.extend(match node {
        CallLikeNode::Call(c) => match c {
            Call::Function(c) => vec![c.function.format(f)],
            Call::Method(c) => vec![c.object.format(f), Document::String("->"), c.method.format(f)],
            Call::NullSafeMethod(c) => vec![c.object.format(f), Document::String("?->"), c.method.format(f)],
            Call::StaticMethod(c) => vec![c.class.format(f), Document::String("::"), c.method.format(f)],
        },
        CallLikeNode::Instantiation(i) => vec![i.new.format(f), Document::space(), i.class.format(f)],
        CallLikeNode::Attribute(a) => vec![a.name.format(f)],
        CallLikeNode::DieConstruct(d) => vec![d.die.format(f)],
        CallLikeNode::ExitConstruct(e) => vec![e.exit.format(f)],
    });

    parts.push(print_call_arguments(f, &node));

    Document::Group(Group::new(parts))
}
