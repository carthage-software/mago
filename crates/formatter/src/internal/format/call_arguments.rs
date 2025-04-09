use mago_span::*;
use mago_syntax::ast::*;

use crate::document::*;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::call_node::CallLikeNode;
use crate::internal::format::misc;
use crate::internal::format::misc::is_simple_expression;
use crate::internal::format::misc::is_string_word_type;
use crate::internal::format::misc::should_hug_expression;
use crate::internal::utils::could_expand_value;
use crate::internal::utils::will_break;

pub(super) fn print_call_arguments<'a>(f: &mut FormatterState<'a>, expression: CallLikeNode<'a>) -> Document<'a> {
    let Some(argument_list) = expression.arguments() else {
        return if (expression.is_instantiation() && f.settings.parentheses_in_new_expression)
            || (expression.is_exit_or_die_construct() && f.settings.parentheses_in_exit_and_die)
            || (expression.is_attribute() && f.settings.parentheses_in_attribute)
        {
            let mut contents = vec![];
            if f.settings.space_before_argument_list_parenthesis {
                contents.push(Document::String(" "));
            }
            contents.push(Document::String("("));
            if f.settings.space_within_argument_list_parenthesis {
                contents.push(Document::space());
            }
            contents.push(Document::String(")"));

            Document::Array(contents)
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
                if f.settings.space_before_argument_list_parenthesis {
                    Document::String(" (")
                } else {
                    Document::String("(")
                },
                inner_comments,
                Document::String(")"),
            ])
        } else {
            Document::empty()
        };
    }

    print_argument_list(f, argument_list, expression.is_attribute())
}

pub(super) fn print_argument_list<'a>(
    f: &mut FormatterState<'a>,
    argument_list: &'a ArgumentList,
    for_attribute: bool,
) -> Document<'a> {
    let left_parenthesis = {
        let mut contents = vec![if f.settings.space_before_argument_list_parenthesis {
            Document::String(" (")
        } else {
            Document::String("(")
        }];

        if let Some(trailing_comments) = f.print_trailing_comments(argument_list.left_parenthesis) {
            contents.push(trailing_comments);
        } else if f.settings.space_within_argument_list_parenthesis {
            contents.push(Document::space());
        }

        Document::Array(contents)
    };

    let get_right_parenthesis = |f: &mut FormatterState<'a>, can_have_space: bool| {
        let mut contents = vec![];
        if let Some(leading_comments) = f.print_leading_comments(argument_list.right_parenthesis) {
            contents.push(leading_comments);
        } else if can_have_space && f.settings.space_within_argument_list_parenthesis {
            contents.push(Document::space());
        }

        contents.push(Document::String(")"));

        Document::Array(contents)
    };

    let mut contents = vec![left_parenthesis.clone()];

    if argument_list.arguments.is_empty() {
        if let Some(inner_comments) = f.print_inner_comment(argument_list.span(), true) {
            contents.push(inner_comments);
        }
        contents.push(get_right_parenthesis(f, false));

        return Document::Array(contents);
    }

    // First, run all the decision functions with unformatted arguments
    let should_break_all = should_break_all_arguments(f, argument_list, for_attribute);
    let should_inline = should_inline_single_breaking_argument(f, argument_list);
    let should_expand_first = should_expand_first_arg(f, argument_list, false);
    let should_expand_last = should_expand_last_arg(f, argument_list, false);
    let is_single_late_breaking_argument = is_single_late_breaking_argument(f, argument_list);

    let arguments_count = argument_list.arguments.len();
    let mut formatted_arguments: Vec<Document<'a>> = argument_list
        .arguments
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            if !should_break_all && !should_inline {
                if should_expand_first && (i == 0) {
                    let previous = f.argument_state.expand_first_argument;
                    f.argument_state.expand_first_argument = true;
                    let document = arg.format(f);
                    f.argument_state.expand_first_argument = previous;

                    return document;
                }

                if should_expand_last && (i == arguments_count - 1) {
                    let previous = f.argument_state.expand_last_argument;
                    f.argument_state.expand_last_argument = true;
                    let document = arg.format(f);
                    f.argument_state.expand_last_argument = previous;

                    return document;
                }
            }

            arg.format(f)
        })
        .collect();

    let get_printed_arguments = |f: &mut FormatterState<'a>, should_break: bool, skip_index: isize| {
        let mut printed_arguments = vec![];
        let mut length = argument_list.arguments.len();
        let arguments_range: Box<dyn Iterator<Item = (usize, usize)>> = match skip_index {
            _ if skip_index > 0 => {
                length -= skip_index as usize;
                Box::new((skip_index as usize..argument_list.arguments.len()).enumerate())
            }
            _ if skip_index < 0 => {
                length -= (-skip_index) as usize;
                Box::new((0..argument_list.arguments.len() - (-skip_index) as usize).enumerate())
            }
            _ => Box::new((0..argument_list.arguments.len()).enumerate()),
        };

        for (i, arg_idx) in arguments_range {
            let element = &argument_list.arguments.as_slice()[arg_idx];
            let mut argument = vec![formatted_arguments[arg_idx].clone()];
            if i < (length - 1) {
                argument.push(Document::String(","));

                if f.is_next_line_empty(element.span()) {
                    argument.push(Document::Line(Line::hard()));
                    argument.push(Document::Line(Line::hard()));
                    argument.push(Document::BreakParent);
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

    let all_arguments_broken_out = |f: &mut FormatterState<'a>| {
        let mut parts = vec![];
        parts.push(left_parenthesis.clone());
        parts.push(Document::Indent(vec![
            Document::Line(Line::hard()),
            Document::Group(Group::new(get_printed_arguments(f, true, 0))),
            if f.settings.trailing_comma { Document::String(",") } else { Document::empty() },
        ]));

        parts.push(Document::Line(Line::hard()));
        if let Some(leading_comments) = f.print_leading_comments(argument_list.right_parenthesis) {
            parts.push(leading_comments);
        }
        parts.push(get_right_parenthesis(f, false));

        Document::Group(Group::new(parts))
    };

    if should_break_all {
        return all_arguments_broken_out(f);
    }

    if is_single_late_breaking_argument {
        let single_argument = formatted_arguments.remove(0);
        let right_parenthesis = get_right_parenthesis(f, false);

        return Document::IfBreak(IfBreak::new(
            Document::Group(Group::new(vec![
                left_parenthesis.clone(),
                Document::IndentIfBreak(IndentIfBreak::new(vec![
                    Document::Line(Line::default()),
                    Document::Group(Group::new(vec![single_argument.clone()])),
                    if f.settings.trailing_comma { Document::String(",") } else { Document::empty() },
                ])),
                Document::Line(Line::default()),
                right_parenthesis.clone(),
            ])),
            Document::Group(Group::new(vec![
                left_parenthesis,
                single_argument,
                if f.settings.space_within_argument_list_parenthesis { Document::space() } else { Document::empty() },
                right_parenthesis,
            ])),
        ));
    }

    if should_inline {
        // we have a single argument that we can hug
        // this means we can avoid any spacing and just print the argument
        // between the parentheses
        let single_argument = formatted_arguments.remove(0);

        return Document::Group(Group::new(vec![
            left_parenthesis,
            Document::Group(Group::new(vec![single_argument])),
            get_right_parenthesis(f, true),
        ]));
    }

    if should_expand_first {
        let mut first_doc = formatted_arguments[0].clone();

        if will_break(&mut first_doc) {
            let last_doc = get_printed_arguments(f, false, 1).pop().unwrap();

            return Document::Array(vec![
                Document::BreakParent,
                Document::Group(Group::conditional(
                    vec![
                        left_parenthesis.clone(),
                        Document::Group(Group::new(vec![first_doc]).with_break(true)),
                        Document::String(", "),
                        last_doc,
                        get_right_parenthesis(f, true),
                    ],
                    vec![all_arguments_broken_out(f)],
                )),
            ]);
        }
    }

    if should_expand_last {
        let mut printed_arguments = get_printed_arguments(f, false, -1);
        let original_printed_arguments = printed_arguments.clone();
        if printed_arguments.iter_mut().any(will_break) {
            return all_arguments_broken_out(f);
        }

        if !printed_arguments.is_empty() {
            printed_arguments.push(Document::String(","));
            printed_arguments.push(Document::Line(Line::default()));
        }

        let last_doc = formatted_arguments.last().unwrap().clone();
        let mut last_doc_clone = last_doc.clone();

        if will_break(&mut last_doc_clone) {
            return Document::Array(vec![
                Document::BreakParent,
                Document::Group(Group::conditional(
                    vec![
                        left_parenthesis.clone(),
                        Document::Array(printed_arguments),
                        Document::Group(Group::new(vec![last_doc]).with_break(true)),
                        Document::String(")"),
                    ],
                    vec![all_arguments_broken_out(f)],
                )),
            ]);
        }

        let right_parenthesis = get_right_parenthesis(f, true);

        return Document::Group(Group::conditional(
            vec![
                left_parenthesis.clone(),
                Document::Array(printed_arguments),
                last_doc.clone(),
                right_parenthesis.clone(),
            ],
            vec![
                Document::Array(vec![
                    left_parenthesis.clone(),
                    if argument_list.arguments.len() > 1 {
                        Document::Array(vec![
                            Document::Array(original_printed_arguments),
                            Document::String(","),
                            Document::Line(Line::default()),
                        ])
                    } else {
                        Document::empty()
                    },
                    Document::Group(Group::new(vec![last_doc]).with_break(true)),
                    right_parenthesis,
                ]),
                all_arguments_broken_out(f),
            ],
        ));
    }

    let mut printed_arguments = get_printed_arguments(f, false, 0);

    printed_arguments.insert(0, Document::Line(Line::soft()));
    contents.push(Document::IndentIfBreak(IndentIfBreak::new(printed_arguments)));
    if f.settings.trailing_comma {
        contents.push(Document::IfBreak(IfBreak::then(Document::String(","))));
    }
    contents.push(if f.settings.space_within_argument_list_parenthesis {
        Document::Line(Line::default())
    } else {
        Document::Line(Line::soft())
    });
    contents.push(get_right_parenthesis(f, false));

    Document::Group(Group::new(contents))
}

#[inline]
fn argument_has_surrounding_comments(f: &FormatterState, argument: &Argument) -> bool {
    f.has_comment(argument.span(), CommentFlags::Leading | CommentFlags::Trailing)
        || f.has_comment(argument.span(), CommentFlags::Leading | CommentFlags::Trailing)
}

#[inline]
fn should_break_all_arguments(f: &FormatterState, argument_list: &ArgumentList, for_attributes: bool) -> bool {
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

    false
}

#[inline]
fn is_single_late_breaking_argument<'a>(f: &FormatterState<'a>, argument_list: &'a ArgumentList) -> bool {
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

    if is_simple_expression(&arrow_function.expression) {
        return true;
    }

    let Expression::Call(call) = arrow_function.expression.as_ref() else {
        return false;
    };

    call.get_argument_list().arguments.iter().all(|a| a.is_positional() && is_simple_expression(a.value()))
}

#[inline]
fn should_inline_single_breaking_argument<'a>(f: &FormatterState<'a>, argument_list: &'a ArgumentList) -> bool {
    let arguments = argument_list.arguments.as_slice();
    if arguments.len() != 1 {
        return false;
    }

    let argument = &arguments[0];

    !argument_has_surrounding_comments(f, argument) && should_hug_expression(f, argument.value())
}

/// * Reference <https://github.com/prettier/prettier/blob/3.3.3/src/language-js/print/call-arguments.js#L247-L272>
pub fn should_expand_first_arg<'a>(f: &FormatterState<'a>, argument_list: &'a ArgumentList, nested_args: bool) -> bool {
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

    could_expand_value(f, first_argument.value(), false, nested_args)
        && (is_hopefully_short_call_argument(second_argument.value())
            && !could_expand_value(f, second_argument.value(), false, nested_args))
}

/// * Reference <https://github.com/prettier/prettier/blob/52829385bcc4d785e58ae2602c0b098a643523c9/src/language-js/print/call-arguments.js#L234-L258>
pub fn should_expand_last_arg<'a>(f: &FormatterState<'a>, argument_list: &'a ArgumentList, nested_args: bool) -> bool {
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

    let penultimate_argument_comments = penultimate_argument
        .map(|a| f.has_comment(a.span(), CommentFlags::Leading | CommentFlags::Trailing))
        .unwrap_or(false);

    could_expand_value(f, last_argument_value, false, nested_args)
        // If the last two arguments are of the same type,
        // disable last element expansion.
        && (penultimate_argument.is_none()
            || penultimate_argument_comments
            || matches!(penultimate_argument, Some(argument) if argument.value().node_kind() != last_argument_value.node_kind()))
        && (argument_list.arguments.len() != 2
            || penultimate_argument_comments
            || !matches!(last_argument_value, Expression::Array(_) | Expression::LegacyArray(_))
            || !matches!(penultimate_argument.map(|a| a.value()), Some(Expression::Closure(c)) if c.use_clause.is_none()))
        && (argument_list.arguments.len() != 2
            || penultimate_argument_comments
            || !matches!(penultimate_argument.map(|a| a.value()), Some(Expression::Array(_) | Expression::LegacyArray(_)))
            || !matches!(last_argument_value, Expression::Closure(c) if c.use_clause.is_none())
        )
}

fn is_hopefully_short_call_argument(mut node: &Expression) -> bool {
    loop {
        node = match node {
            Expression::Parenthesized(parenthesized) => &parenthesized.expression,
            Expression::UnaryPrefix(operation) if !operation.operator.is_cast() => operation.operand.as_ref(),
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
            instantiation.arguments.as_ref().is_none_or(|argument_list| argument_list.arguments.len() < 2)
        }
        Expression::Binary(operation) => {
            is_simple_call_argument(&operation.lhs, 1) && is_simple_call_argument(&operation.rhs, 1)
        }
        _ => is_simple_call_argument(node, 2),
    }
}

fn is_simple_call_argument<'a>(node: &'a Expression, depth: usize) -> bool {
    let is_child_simple = |node: &'a Expression| {
        if depth <= 1 {
            return false;
        }

        is_simple_call_argument(node, depth - 1)
    };

    let is_simple_element = |node: &'a ArrayElement| match node {
        ArrayElement::KeyValue(element) => is_child_simple(&element.key) && is_child_simple(&element.value),
        ArrayElement::Value(element) => is_child_simple(&element.value),
        ArrayElement::Variadic(element) => is_child_simple(&element.value),
        ArrayElement::Missing(_) => true,
    };

    if node.is_literal() || is_string_word_type(node) {
        return true;
    }

    match node {
        Expression::Parenthesized(parenthesized) => is_simple_call_argument(&parenthesized.expression, depth),
        Expression::UnaryPrefix(operation) => {
            if let UnaryPrefixOperator::PreIncrement(_) | UnaryPrefixOperator::PreDecrement(_) = operation.operator {
                return false;
            }

            if operation.operator.is_cast() {
                return false;
            }

            is_simple_call_argument(&operation.operand, depth)
        }
        Expression::Array(array) => array.elements.iter().all(is_simple_element),
        Expression::LegacyArray(array) => array.elements.iter().all(is_simple_element),
        Expression::Call(call) => {
            let argument_list = match call {
                Call::Function(function_call) => {
                    if !is_simple_call_argument(&function_call.function, depth) {
                        return false;
                    }

                    &function_call.argument_list
                }
                Call::Method(method_call) => {
                    if !is_simple_call_argument(&method_call.object, depth) {
                        return false;
                    }

                    &method_call.argument_list
                }
                Call::NullSafeMethod(null_safe_method_call) => {
                    if !is_simple_call_argument(&null_safe_method_call.object, depth) {
                        return false;
                    }

                    &null_safe_method_call.argument_list
                }
                Call::StaticMethod(static_method_call) => {
                    if !is_simple_call_argument(&static_method_call.class, depth) {
                        return false;
                    }

                    &static_method_call.argument_list
                }
            };

            argument_list.arguments.len() <= depth
                && argument_list.arguments.iter().map(|a| a.value()).all(is_child_simple)
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
            is_simple_call_argument(&array_access.array, depth) && is_simple_call_argument(&array_access.index, depth)
        }
        Expression::Instantiation(instantiation) => {
            if is_simple_call_argument(&instantiation.class, depth) {
                match &instantiation.arguments {
                    Some(argument_list) => {
                        argument_list.arguments.len() <= depth
                            && argument_list.arguments.iter().map(|a| a.value()).all(is_child_simple)
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
