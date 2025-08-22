use bumpalo::vec;

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

pub(super) enum CallLikeNode<'arena> {
    Call(&'arena Call<'arena>),
    Instantiation(&'arena Instantiation<'arena>),
    Attribute(&'arena Attribute<'arena>),
    DieConstruct(&'arena DieConstruct<'arena>),
    ExitConstruct(&'arena ExitConstruct<'arena>),
}

impl<'arena> CallLikeNode<'arena> {
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

    pub fn arguments(&self) -> Option<&'arena ArgumentList<'arena>> {
        match self {
            CallLikeNode::Call(call) => Some(match call {
                Call::Function(c) => &c.argument_list,
                Call::Method(c) => &c.argument_list,
                Call::NullSafeMethod(c) => &c.argument_list,
                Call::StaticMethod(c) => &c.argument_list,
            }),
            CallLikeNode::Instantiation(new) => new.argument_list.as_ref(),
            CallLikeNode::Attribute(attr) => attr.argument_list.as_ref(),
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

pub(super) fn print_call_like_node<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    node: CallLikeNode<'arena>,
) -> Document<'arena> {
    // format the callee-like expression
    let mut parts = match node {
        CallLikeNode::Call(c) => match c {
            Call::Function(c) => vec![in f.arena; c.function.format(f)],
            Call::StaticMethod(c) => vec![in f.arena; c.class.format(f), Document::String("::"), c.method.format(f)],
            _ => {
                return print_access_call_node(f, c);
            }
        },
        CallLikeNode::Instantiation(i) => vec![in f.arena; i.new.format(f), Document::space(), i.class.format(f)],
        CallLikeNode::Attribute(a) => vec![in f.arena; a.name.format(f)],
        CallLikeNode::DieConstruct(d) => vec![in f.arena; d.die.format(f)],
        CallLikeNode::ExitConstruct(e) => vec![in f.arena; e.exit.format(f)],
    };

    parts.push(print_call_arguments(f, node));

    Document::Group(Group::new(parts))
}

fn print_access_call_node<'arena>(f: &mut FormatterState<'_, 'arena>, node: &'arena Call<'arena>) -> Document<'arena> {
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

    let base_span = base.span();
    let should_break = f.has_inner_comment(Span::new(base_span.file_id, base_span.end, operator.start))
        || (f.settings.preserve_breaking_member_access_chain
            && misc::has_new_line_in_range(f.source_text, base_span.end.offset, operator.start.offset));

    if should_break {
        Document::Group(Group::new(vec![
            in f.arena;
            base.format(f),
            Document::Line(Line::hard()),
            format_access_operator(f, operator, operator_str),
            selector.format(f),
            print_call_arguments(f, CallLikeNode::Call(node)),
        ]))
    } else {
        Document::Group(Group::new(vec![
            in f.arena;
            base.format(f),
            format_access_operator(f, operator, operator_str),
            selector.format(f),
            print_call_arguments(f, CallLikeNode::Call(node)),
        ]))
    }
}
