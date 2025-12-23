use bumpalo::collections::Vec;
use bumpalo::vec;

use mago_span::HasSpan;
use mago_syntax::ast::Access;
use mago_syntax::ast::Argument;
use mago_syntax::ast::ArgumentList;
use mago_syntax::ast::ArrayElement;
use mago_syntax::ast::Call;
use mago_syntax::ast::Expression;
use mago_syntax::ast::PartialArgumentList;
use mago_syntax::ast::UnaryPrefixOperator;

use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::document::Separator;
use crate::document::clone_in_arena;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::call_node::CallLikeNode;
use crate::internal::format::format_token;
use crate::internal::format::misc;
use crate::internal::format::misc::is_breaking_expression;
use crate::internal::format::misc::is_expandable_expression;
use crate::internal::format::misc::is_simple_expression;
use crate::internal::format::misc::is_simple_single_line_expression;
use crate::internal::format::misc::is_string_word_type;
use crate::internal::format::misc::should_hug_expression;
use crate::internal::utils::could_expand_value;
use crate::internal::utils::foreach_binary_operand;
use crate::internal::utils::get_expression_width;
use crate::internal::utils::unwrap_parenthesized;
use crate::internal::utils::will_break;

pub(super) fn print_call_arguments<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    expression: CallLikeNode<'arena>,
) -> Document<'arena> {
    let Some(argument_list) = expression.arguments() else {
        return if (expression.is_instantiation() && f.settings.parentheses_in_new_expression)
            || (expression.is_exit_or_die_construct() && f.settings.parentheses_in_exit_and_die)
            || (expression.is_attribute() && f.settings.parentheses_in_attribute)
        {
            Document::String("()")
        } else {
            Document::empty()
        };
    };

    if argument_list.arguments.is_empty()
        && ((expression.is_instantiation() && !f.settings.parentheses_in_new_expression)
            || (expression.is_exit_or_die_construct() && !f.settings.parentheses_in_exit_and_die)
            || (expression.is_attribute() && !f.settings.parentheses_in_attribute))
    {
        return if let Some(inner_comments) = f.print_inner_comment(argument_list.span(), true) {
            Document::Array(vec![
                in f.arena;
                Document::String("("),
                inner_comments,
                Document::String(")"),
            ])
        } else {
            Document::empty()
        };
    }

    print_argument_list(f, argument_list, expression.is_attribute(), !expression.is_phpunit_assertion_call())
}

pub(super) fn print_argument_list<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    argument_list: &'arena ArgumentList<'arena>,
    for_attribute: bool,
    can_expand_first_or_last: bool,
) -> Document<'arena> {
    let mut force_break = false;
    let left_parenthesis = {
        let mut contents = vec![
            in f.arena;
            Document::String("(")
        ];

        if let Some(trailing_comment) = f.print_trailing_comments(argument_list.left_parenthesis) {
            contents.push(trailing_comment);
            force_break = true;
        }

        if let Some(argument) = argument_list.arguments.first()
            && let Some(trailing_comments) =
                f.print_dangling_comments_between_nodes(argument_list.left_parenthesis, argument.span())
        {
            contents.push(trailing_comments);
            force_break = true;
        }

        Document::Array(contents)
    };

    if !force_break {
        let args = argument_list.arguments.as_slice();
        for (i, arg) in args.iter().enumerate() {
            if i < args.len() - 1 && f.has_comment(arg.span(), CommentFlags::Trailing | CommentFlags::Line) {
                force_break = true;
                break;
            }
        }
    }

    let mut contents = vec![in f.arena; clone_in_arena(f.arena, &left_parenthesis)];

    // First, run all the decision functions with unformatted arguments
    let should_break_all = force_break || should_break_all_arguments(f, argument_list, for_attribute);
    let should_inline = !force_break && should_inline_breaking_arguments(f, argument_list);
    let should_expand_first =
        can_expand_first_or_last && !force_break && should_expand_first_arg(f, argument_list, false);
    let should_expand_last =
        can_expand_first_or_last && !force_break && should_expand_last_arg(f, argument_list, false);
    let is_single_late_breaking_argument = !force_break && is_single_late_breaking_argument(f, argument_list);

    let arguments_count = argument_list.arguments.len();
    let mut formatted_arguments: Vec<'arena, Document<'arena>> = Vec::with_capacity_in(arguments_count, f.arena);
    for (i, arg) in argument_list.arguments.iter().enumerate() {
        if !should_break_all && !should_inline {
            if should_expand_first && (i == 0) {
                let previous = f.argument_state.expand_first_argument;
                f.argument_state.expand_first_argument = true;
                let document = arg.format(f);
                f.argument_state.expand_first_argument = previous;

                formatted_arguments.push(document);
                continue;
            }

            if should_expand_last && (i == arguments_count - 1) {
                let previous = f.argument_state.expand_last_argument;
                f.argument_state.expand_last_argument = true;
                let document = arg.format(f);
                f.argument_state.expand_last_argument = previous;

                formatted_arguments.push(document);
                continue;
            }
        }

        formatted_arguments.push(arg.format(f));
    }

    let dangling_comments = f.print_dangling_comments(argument_list.span(), true);
    let right_parenthesis = format_token(f, argument_list.right_parenthesis, ")");

    if arguments_count == 0 {
        contents.push(print_right_parenthesis(f, dangling_comments.as_ref(), &right_parenthesis, Some(false)));

        return Document::Array(contents);
    }

    let get_printed_arguments = |f: &mut FormatterState<'_, 'arena>, should_break: bool, skip_index: isize| {
        let mut printed_arguments = vec![in f.arena];
        let mut length = arguments_count;
        let (arguments_start, arguments_end) = match skip_index {
            _ if skip_index > 0 => {
                length -= skip_index as usize;
                (skip_index as usize, arguments_count)
            }
            _ if skip_index < 0 => {
                length -= (-skip_index) as usize;
                (0, arguments_count - (-skip_index) as usize)
            }
            _ => (0, arguments_count),
        };

        for (i, arg_idx) in (arguments_start..arguments_end).enumerate() {
            let element = &argument_list.arguments.as_slice()[arg_idx];
            let mut argument = vec![in f.arena; clone_in_arena(f.arena, &formatted_arguments[arg_idx])];
            if i < (length - 1) {
                argument.push(Document::String(","));

                if f.is_next_line_empty(element.span()) {
                    argument.push(Document::Line(Line::hard()));
                    argument.push(Document::Line(Line::hard()));
                } else if should_break {
                    argument.push(Document::Line(Line::hard()));
                } else {
                    argument.push(Document::Line(Line::default()));
                }
            }

            printed_arguments.push(Document::Array(argument));
        }

        printed_arguments
    };

    let all_arguments_broken_out = |f: &mut FormatterState<'_, 'arena>| {
        let mut parts = vec![in f.arena];
        parts.push(clone_in_arena(f.arena, &left_parenthesis));
        parts.push(Document::Indent(vec![
            in f.arena;
            Document::Line(Line::hard()),
            Document::Group(Group::new(get_printed_arguments(f, true, 0))),
            if f.settings.trailing_comma { Document::String(",") } else { Document::empty() },
        ]));

        parts.push(print_right_parenthesis(f, dangling_comments.as_ref(), &right_parenthesis, Some(true)));

        Document::Group(Group::new(parts).with_break(true))
    };

    if should_break_all {
        return all_arguments_broken_out(f);
    }

    if is_single_late_breaking_argument {
        let single_argument = formatted_arguments.remove(0);
        let right_parenthesis = print_right_parenthesis(f, dangling_comments.as_ref(), &right_parenthesis, None);

        let group_id = f.next_id();

        return Document::IfBreak(IfBreak::new(
            f.arena,
            Document::Group(
                Group::new(vec![
                    in f.arena;
                    clone_in_arena(f.arena, &left_parenthesis),
                    Document::IndentIfBreak(IndentIfBreak::new(group_id, vec![
                        in f.arena;
                        Document::Line(Line::soft()),
                        Document::Group(Group::new(vec![in f.arena; clone_in_arena(f.arena, &single_argument)])),
                    ])),
                    if f.settings.trailing_comma {
                        Document::IfBreak(IfBreak::new(f.arena, Document::String(","), Document::empty()))
                    } else {
                        Document::empty()
                    },
                    clone_in_arena(f.arena, &right_parenthesis)
                ])
                .with_id(group_id),
            ),
            Document::Group(Group::new(vec![in f.arena; left_parenthesis, single_argument, right_parenthesis])),
        ));
    }

    if should_inline {
        return Document::Group(Group::new(vec![
            in f.arena;
            left_parenthesis,
            Document::Group(Group::new(Document::join(f.arena, formatted_arguments, Separator::CommaSpace))),
            print_right_parenthesis(f, dangling_comments.as_ref(), &right_parenthesis, Some(false)),
        ]));
    }

    if should_expand_first
        && let Some(first_argument) = formatted_arguments.first()
        && let Some(last_argument) = formatted_arguments.last()
    {
        if will_break(last_argument) {
            return all_arguments_broken_out(f);
        }

        let first_argument = clone_in_arena(f.arena, first_argument);
        let last_argument = clone_in_arena(f.arena, last_argument);

        if will_break(&first_argument) {
            return Document::Group(Group::new(vec![
                in f.arena;
                Document::Group(Group::conditional(
                    vec![
                        in f.arena;
                        clone_in_arena(f.arena, &left_parenthesis),
                        Document::Group(Group::new(vec![in f.arena; first_argument]).with_break(true)),
                        Document::String(", "),
                        last_argument,
                        print_right_parenthesis(f, dangling_comments.as_ref(), &right_parenthesis, None),
                    ],
                    vec![in f.arena; all_arguments_broken_out(f)],
                )),
            ]));
        }

        return Document::Group(Group::new(vec![
            in f.arena;
            Document::Group(Group::conditional(
                vec![
                    in f.arena;
                    clone_in_arena(f.arena, &left_parenthesis),
                    clone_in_arena(f.arena, &first_argument),
                    Document::String(", "),
                    clone_in_arena(f.arena, &last_argument),
                    print_right_parenthesis(f, dangling_comments.as_ref(), &right_parenthesis, None),
                ],
                vec![
                    in f.arena;
                    Document::Array(vec![
                        in f.arena;
                        clone_in_arena(f.arena, &left_parenthesis),
                        Document::Group(Group::new(vec![in f.arena; first_argument]).with_break(true)),
                        Document::String(", "),
                        last_argument,
                        print_right_parenthesis(f, dangling_comments.as_ref(), &right_parenthesis, None),
                    ]),
                    all_arguments_broken_out(f),
                ],
            )),
        ]));
    }

    if should_expand_last {
        let mut first_arguments = get_printed_arguments(f, false, -1);
        if first_arguments.iter().any(will_break) {
            return all_arguments_broken_out(f);
        }

        if !first_arguments.is_empty() {
            first_arguments.push(Document::String(","));
            first_arguments.push(Document::Line(Line::default()));
        }

        let last_argument = clone_in_arena(f.arena, formatted_arguments.last().unwrap());

        return Document::Group(Group::new(vec![
            in f.arena;
            Document::Group(Group::conditional(
                vec![
                    in f.arena;
                    clone_in_arena(f.arena, &left_parenthesis),
                    Document::Array(first_arguments),
                    Document::Group(Group::new(vec![in f.arena; last_argument]).with_break(true)),
                    print_right_parenthesis(f, dangling_comments.as_ref(), &right_parenthesis, None),
                ],
                vec![
                    in f.arena;
                    all_arguments_broken_out(f),
                ],
            )),
        ]));
    }

    let mut printed_arguments = get_printed_arguments(f, false, 0);

    let group_id = f.next_id();
    printed_arguments.insert(0, Document::Line(Line::soft()));
    contents.push(Document::IndentIfBreak(IndentIfBreak::new(group_id, printed_arguments)));
    if f.settings.trailing_comma {
        contents.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(","))));
    }
    contents.push(print_right_parenthesis(f, dangling_comments.as_ref(), &right_parenthesis, None));

    Document::Group(Group::new(contents).with_id(group_id))
}

fn print_right_parenthesis<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    dangling_comments: Option<&Document<'arena>>,
    right_parenthesis: &Document<'arena>,
    breaking: Option<bool>,
) -> Document<'arena> {
    let mut contents = vec![in f.arena];

    if let Some(dangling) = dangling_comments {
        contents.push(clone_in_arena(f.arena, dangling));
    } else {
        match breaking {
            Some(true) => {
                contents.push(Document::Line(Line::hard()));
            }
            None => {
                contents.push(Document::Line(Line::soft()));
            }
            _ => { /* nothing */ }
        }
    }

    contents.push(clone_in_arena(f.arena, right_parenthesis));

    Document::Array(contents)
}

#[inline]
fn argument_has_surrounding_comments(f: &FormatterState, argument: &Argument) -> bool {
    f.has_comment(argument.span(), CommentFlags::Leading | CommentFlags::Trailing)
        || f.has_comment(argument.span(), CommentFlags::Leading | CommentFlags::Trailing)
}

pub(super) fn print_partial_argument_list<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    argument_list: &'arena PartialArgumentList<'arena>,
    _for_attribute: bool,
    _can_expand_first_or_last: bool,
) -> Document<'arena> {
    let left_parenthesis = Document::String("(");
    let mut contents = vec![in f.arena; left_parenthesis];

    let arguments_count = argument_list.arguments.len();
    let mut formatted_arguments: Vec<'arena, Document<'arena>> = Vec::with_capacity_in(arguments_count, f.arena);
    for arg in &argument_list.arguments {
        formatted_arguments.push(arg.format(f));
    }

    let dangling_comments = f.print_dangling_comments(argument_list.span(), true);
    let right_parenthesis = format_token(f, argument_list.right_parenthesis, ")");

    if arguments_count == 0 {
        // Technically, this is impossible, but just in case..
        contents.push(print_right_parenthesis(f, dangling_comments.as_ref(), &right_parenthesis, Some(false)));
        return Document::Array(contents);
    }

    let get_printed_arguments = |f: &mut FormatterState<'_, 'arena>, should_break: bool| {
        let mut printed_arguments = vec![in f.arena];

        for (i, arg_idx) in (0..arguments_count).enumerate() {
            let element = &argument_list.arguments.as_slice()[arg_idx];
            let mut argument = vec![in f.arena; clone_in_arena(f.arena, &formatted_arguments[arg_idx])];
            if i < (arguments_count - 1) {
                argument.push(Document::String(","));

                if f.is_next_line_empty(element.span()) {
                    argument.push(Document::Line(Line::hard()));
                    argument.push(Document::Line(Line::hard()));
                } else if should_break {
                    argument.push(Document::Line(Line::hard()));
                } else {
                    argument.push(Document::Line(Line::default()));
                }
            }

            printed_arguments.push(Document::Array(argument));
        }

        printed_arguments
    };

    let printed_args = get_printed_arguments(f, false);
    contents.push(Document::Group(Group::new(printed_args)));
    contents.push(print_right_parenthesis(f, dangling_comments.as_ref(), &right_parenthesis, None));

    Document::Group(Group::new(contents))
}

/// Checks if an expression is a concatenation chain (3+ operands) that exceeds print width.
/// This is used to determine if the parent call should break.
pub fn is_breaking_binary(f: &FormatterState, expr: &Expression) -> bool {
    let threshold = f.settings.print_width + f.settings.tab_width;

    let mut estimated_total_width = 0;
    foreach_binary_operand(expr, &mut |operand: &Expression| {
        estimated_total_width += 3; // for the operator and spaces
        estimated_total_width += get_expression_width(operand).unwrap_or(0);
    });

    estimated_total_width > threshold
}

#[inline]
pub fn should_break_all_arguments(f: &FormatterState, argument_list: &ArgumentList, for_attributes: bool) -> bool {
    if f.settings.always_break_named_arguments_list
        && (!for_attributes || f.settings.always_break_attribute_named_argument_lists)
        && argument_list.arguments.len() >= 2
        && argument_list.arguments.iter().all(|a| matches!(a, Argument::Named(_)))
    {
        return true;
    }

    if f.settings.preserve_breaking_argument_list
        && !argument_list.arguments.is_empty()
        && misc::has_new_line_in_range(
            f.source_text,
            argument_list.left_parenthesis.start.offset,
            argument_list.arguments.as_slice()[0].span().start.offset,
        )
    {
        return true;
    }

    if argument_list.arguments.len() >= 2
        && argument_list.arguments.iter().any(|a| is_call_with_wrapping_arrow_function(a.value()))
    {
        return true;
    }

    if argument_list.arguments.len() == 1
        && let Some(arg) = argument_list.arguments.first()
        && is_breaking_binary(f, arg.value())
    {
        return true;
    }

    false
}

#[inline]
fn is_single_late_breaking_argument<'arena>(
    f: &FormatterState<'_, 'arena>,
    argument_list: &'arena ArgumentList<'arena>,
) -> bool {
    let arguments = argument_list.arguments.as_slice();
    if arguments.len() != 1 {
        return false;
    }

    let argument = &arguments[0];
    if !argument.is_positional() && argument_has_surrounding_comments(f, argument) {
        return false;
    }

    let Expression::ArrowFunction(arrow_function) = argument.value() else {
        return false;
    };

    if is_simple_expression(arrow_function.expression) {
        return true;
    }

    let Expression::Call(call) = arrow_function.expression else {
        return false;
    };

    call.get_argument_list().arguments.iter().all(|a| a.is_positional() && is_simple_expression(a.value()))
}

#[inline]
fn should_inline_breaking_arguments<'arena>(
    f: &FormatterState<'_, 'arena>,
    argument_list: &'arena ArgumentList<'arena>,
) -> bool {
    let arguments = argument_list.arguments.as_slice();

    match arguments.len() {
        1 => {
            !argument_has_surrounding_comments(f, &arguments[0])
                && should_hug_expression(f, arguments[0].value(), false)
        }
        2 => {
            let Some(first_argument) = arguments.first() else {
                return false;
            };

            let Some(second_argument) = arguments.last() else {
                return false;
            };

            if argument_has_surrounding_comments(f, first_argument)
                || argument_has_surrounding_comments(f, second_argument)
            {
                return false;
            }

            let first_expression = first_argument.value();
            let second_expression = second_argument.value();

            let is_first_breaking = is_breaking_expression(f, first_expression, false);
            let is_second_breaking = is_breaking_expression(f, second_expression, false);
            if is_first_breaking && is_second_breaking {
                return true;
            }

            if !is_simple_single_line_expression(f, first_expression) {
                return false;
            }

            is_second_breaking
                || could_expand_value(f, second_expression, false)
                || matches!(second_expression, Expression::Call(call) if call.get_argument_list().arguments.len() >= 2)
        }
        _ => false,
    }
}

/// Checks if an expression is a method/function call with an arrow function argument
/// that has a conditional body which will be wrapped in parentheses.
#[inline]
fn is_call_with_wrapping_arrow_function<'arena>(expr: &'arena Expression<'arena>) -> bool {
    let Expression::Call(call) = expr else {
        return false;
    };

    let args = call.get_argument_list().arguments.as_slice();
    if args.len() != 1 {
        return false;
    }

    let Expression::ArrowFunction(arrow) = args[0].value() else {
        return false;
    };

    let body = unwrap_parenthesized(arrow.expression);

    if let Expression::Conditional(conditional) = body {
        if let Some(then) = conditional.then {
            return is_expandable_expression(then, false)
                || is_expandable_expression(conditional.r#else, false)
                || then.is_conditional()
                || conditional.r#else.is_conditional();
        }

        return true;
    }

    if let Expression::Binary(binary) = body {
        return is_expandable_expression(binary.lhs, false) || is_expandable_expression(binary.rhs, false);
    }

    false
}

/// * Reference <https://github.com/prettier/prettier/blob/3.3.3/src/language-js/print/call-arguments.js#L247-L272>
pub fn should_expand_first_arg<'arena>(
    f: &FormatterState<'_, 'arena>,
    argument_list: &'arena ArgumentList<'arena>,
    nested_args: bool,
) -> bool {
    if argument_list.arguments.len() != 2 {
        return false;
    }

    let arguments = argument_list.arguments.as_slice();
    let first_argument = &arguments[0];
    let second_argument = &arguments[1];

    if f.has_comment(first_argument.span(), CommentFlags::Leading | CommentFlags::Trailing)
        || f.has_comment(second_argument.span(), CommentFlags::Leading | CommentFlags::Trailing)
    {
        return false;
    }

    could_expand_value(f, first_argument.value(), nested_args)
        && is_hopefully_short_call_argument(second_argument.value())
        && !could_expand_value(f, second_argument.value(), nested_args)
}

/// * Reference <https://github.com/prettier/prettier/blob/52829385bcc4d785e58ae2602c0b098a643523c9/src/language-js/print/call-arguments.js#L234-L258>
pub fn should_expand_last_arg<'arena>(
    f: &FormatterState<'_, 'arena>,
    argument_list: &'arena ArgumentList<'arena>,
    nested_args: bool,
) -> bool {
    let Some(last_argument) = argument_list.arguments.last() else { return false };
    if f.has_comment(last_argument.span(), CommentFlags::Leading | CommentFlags::Trailing) {
        return false;
    }

    let last_argument_value = last_argument.value();
    let penultimate_argument = if argument_list.arguments.len() >= 2 {
        argument_list.arguments.get(argument_list.arguments.len() - 2)
    } else {
        None
    };

    let penultimate_argument_comments =
        penultimate_argument.is_some_and(|a| f.has_comment(a.span(), CommentFlags::Leading | CommentFlags::Trailing));

    could_expand_value(f, last_argument_value, nested_args)
        // If the last two arguments are of the same type,
        // disable last element expansion.
        && (penultimate_argument.is_none()
            || penultimate_argument_comments
            || matches!(penultimate_argument, Some(argument) if argument.value().node_kind() != last_argument_value.node_kind()))
        && (argument_list.arguments.len() != 2
            || penultimate_argument_comments
            || !matches!(last_argument_value, Expression::Array(_) | Expression::LegacyArray(_))
            || !matches!(penultimate_argument.map(Argument::value), Some(Expression::Closure(c)) if c.use_clause.is_none()))
        && (argument_list.arguments.len() != 2
            || penultimate_argument_comments
            || !matches!(penultimate_argument.map(Argument::value), Some(Expression::Array(_) | Expression::LegacyArray(_)))
            || !matches!(last_argument_value, Expression::Closure(c) if c.use_clause.is_none())
        )
}

fn is_hopefully_short_call_argument(mut node: &Expression) -> bool {
    loop {
        node = match node {
            Expression::Parenthesized(parenthesized) => parenthesized.expression,
            Expression::UnaryPrefix(operation) if !operation.operator.is_cast() => operation.operand,
            Expression::Assignment(assignment) => {
                if !assignment.operator.is_assign() || !is_simple_expression(assignment.lhs) {
                    break;
                }

                assignment.rhs
            }
            _ => break,
        };
    }

    match node {
        Expression::Call(call) => {
            let argument_list = match call {
                Call::Function(function_call) => &function_call.argument_list,
                Call::Method(method_call) => &method_call.argument_list,
                Call::NullSafeMethod(null_safe_method_call) => &null_safe_method_call.argument_list,
                Call::StaticMethod(static_method_call) => &static_method_call.argument_list,
            };

            argument_list.arguments.len() < 2
        }
        Expression::Instantiation(instantiation) => {
            instantiation.argument_list.as_ref().is_none_or(|argument_list| argument_list.arguments.len() < 2)
        }
        Expression::Binary(operation) => {
            is_simple_call_argument(operation.lhs, 1) && is_simple_call_argument(operation.rhs, 1)
        }
        _ => is_simple_call_argument(node, 2),
    }
}

fn is_simple_call_argument<'arena>(node: &'arena Expression<'arena>, depth: usize) -> bool {
    let is_child_simple = |node: &'arena Expression<'arena>| {
        if depth <= 1 {
            return false;
        }

        is_simple_call_argument(node, depth - 1)
    };

    let is_simple_element = |node: &'arena ArrayElement<'arena>| match node {
        ArrayElement::KeyValue(element) => is_child_simple(element.key) && is_child_simple(element.value),
        ArrayElement::Value(element) => is_child_simple(element.value),
        ArrayElement::Variadic(element) => is_child_simple(element.value),
        ArrayElement::Missing(_) => true,
    };

    if node.is_literal() || is_string_word_type(node) {
        return true;
    }

    match node {
        Expression::Parenthesized(parenthesized) => is_simple_call_argument(parenthesized.expression, depth),
        Expression::UnaryPrefix(operation) => {
            if let UnaryPrefixOperator::PreIncrement(_) | UnaryPrefixOperator::PreDecrement(_) = operation.operator {
                return false;
            }

            if operation.operator.is_cast() {
                return false;
            }

            is_simple_call_argument(operation.operand, depth)
        }
        Expression::Array(array) => array.elements.iter().all(is_simple_element),
        Expression::LegacyArray(array) => array.elements.iter().all(is_simple_element),
        Expression::Call(call) => {
            let argument_list = match call {
                Call::Function(function_call) => {
                    if !is_simple_call_argument(function_call.function, depth) {
                        return false;
                    }

                    &function_call.argument_list
                }
                Call::Method(method_call) => {
                    if !is_simple_call_argument(method_call.object, depth) {
                        return false;
                    }

                    &method_call.argument_list
                }
                Call::NullSafeMethod(null_safe_method_call) => {
                    if !is_simple_call_argument(null_safe_method_call.object, depth) {
                        return false;
                    }

                    &null_safe_method_call.argument_list
                }
                Call::StaticMethod(static_method_call) => {
                    if !is_simple_call_argument(static_method_call.class, depth) {
                        return false;
                    }

                    &static_method_call.argument_list
                }
            };

            argument_list.arguments.len() <= depth
                && argument_list.arguments.iter().map(Argument::value).all(is_child_simple)
        }
        Expression::Access(access) => {
            let object_or_class = match access {
                Access::Property(property_access) => &property_access.object,
                Access::NullSafeProperty(null_safe_property_access) => &null_safe_property_access.object,
                Access::StaticProperty(static_property_access) => &static_property_access.class,
                Access::ClassConstant(class_constant_access) => &class_constant_access.class,
            };

            is_simple_call_argument(object_or_class, depth)
        }
        Expression::ArrayAccess(array_access) => {
            is_simple_call_argument(array_access.array, depth) && is_simple_call_argument(array_access.index, depth)
        }
        Expression::Instantiation(instantiation) => {
            if is_simple_call_argument(instantiation.class, depth) {
                match &instantiation.argument_list {
                    Some(argument_list) => {
                        argument_list.arguments.len() <= depth
                            && argument_list.arguments.iter().map(Argument::value).all(is_child_simple)
                    }
                    None => true,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}
