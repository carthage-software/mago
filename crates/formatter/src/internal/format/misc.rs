use bumpalo::collections::Vec;
use bumpalo::vec;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Access;
use mago_syntax::ast::Argument;
use mago_syntax::ast::ArrayAccess;
use mago_syntax::ast::AttributeList;
use mago_syntax::ast::Call;
use mago_syntax::ast::ConstantAccess;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::Instantiation;
use mago_syntax::ast::Keyword;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Modifier;
use mago_syntax::ast::Node;
use mago_syntax::ast::Sequence;
use mago_syntax::ast::Statement;
use mago_syntax::ast::Terminator;
use mago_syntax::ast::Variable;
use mago_syntax::ast::Yield;

use crate::document::Document;
use crate::document::Group;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::document::Separator;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::call_arguments::should_break_all_arguments;
use crate::internal::format::format_token;
use crate::internal::format::member_access::collect_member_access_chain;
use crate::internal::format::statement::print_statement_sequence;
use crate::internal::utils::string_width;
use crate::settings::BraceStyle;

use super::block::block_is_empty;

/// Check if there is a newline character within a specified range of text.
#[inline(always)]
pub(super) fn has_new_line_in_range(text: &str, start: u32, end: u32) -> bool {
    text[start as usize..end as usize].contains('\n')
}

/// Determines whether an expression can be "hugged" within brackets without line breaks.
///
/// # Overview
///
/// A "huggable" expression can be formatted compactly within parentheses `()` or square brackets `[]`
/// without requiring additional line breaks or indentation. This means the expression can be
/// rendered on the same line as the opening and closing brackets.
///
/// # Hugging Rules
///
/// 1. Nested expressions are recursively checked
/// 2. Expressions with leading or trailing comments cannot be hugged
/// 3. Specific expression types are considered huggable
///
/// # Supported Huggable Expressions
///
/// - Arrays
/// - Legacy Arrays
/// - Lists
/// - Closures
/// - Closure Creations
/// - Function Calls
/// - Anonymous Classes
/// - Match Expressions
///
/// # Parameters
///
/// - `f`: The formatter context
/// - `expression`: The expression to check for hugging potential
///
/// # Returns
///
/// `true` if the expression can be formatted compactly, `false` otherwise
///
/// # Performance
///
/// O(1) for most checks, with potential O(n) recursion for nested expressions
pub(super) fn should_hug_expression<'arena>(
    f: &FormatterState<'_, 'arena>,
    expression: &'arena Expression<'arena>,
    arrow_function_recursion: bool,
) -> bool {
    if let Expression::Parenthesized(inner) = expression {
        return should_hug_expression(f, inner.expression, arrow_function_recursion);
    }

    if let Expression::UnaryPrefix(operation) = expression {
        return should_hug_expression(f, operation.operand, arrow_function_recursion);
    }

    // if the expression has leading or trailing comments, we can't hug it
    if f.has_comment(expression.span(), CommentFlags::Leading | CommentFlags::Trailing) {
        return false;
    }

    if is_breaking_expression(f, expression, arrow_function_recursion) {
        return true;
    }

    if let Expression::Call(_) | Expression::Access(_) = expression {
        // Don't hug calls/accesses if they are part of a member access chain
        return collect_member_access_chain(f.arena, expression).is_none_or(|chain| !chain.is_eligible_for_chaining(f));
    }

    if let Expression::ArrowFunction(arrow_function) = expression {
        return !arrow_function_recursion && should_hug_expression(f, arrow_function.expression, true);
    }

    if let Expression::Binary(binary) = expression {
        let is_left_hand_side_simple = is_simple_expression(binary.lhs);
        let is_right_hand_side_simple = is_simple_expression(binary.rhs);

        // Hug binary expressions if they are simple and not too complex
        if is_left_hand_side_simple && is_right_hand_side_simple {
            return true;
        }

        if binary.operator.is_concatenation() {
            if matches!(binary.lhs, Expression::Call(_)) || matches!(binary.rhs, Expression::Call(_)) {
                return false;
            }

            return (is_left_hand_side_simple && should_hug_expression(f, binary.rhs, arrow_function_recursion))
                || (is_right_hand_side_simple && should_hug_expression(f, binary.lhs, arrow_function_recursion));
        }

        return false;
    }

    let Expression::Instantiation(instantiation) = expression else {
        return false;
    };

    // Hug instantiations if it is a simple class instantiation
    let Expression::Identifier(_) = instantiation.class else {
        return false;
    };

    // And either:
    match &instantiation.argument_list {
        // a. The instantiation is a simple class instantiation without arguments
        None => true,
        Some(argument_list) => {
            let arguments_len = argument_list.arguments.len();
            if 0 == arguments_len {
                false
            } else if arguments_len == 1 {
                // b. The instantiation has a single non-named argument that is huggable or an instantiation
                //   (e.g. `new Foo(new Bar())`)
                match &argument_list.arguments.as_slice()[0] {
                    Argument::Named(_) => false,
                    Argument::Positional(positional) => {
                        matches!(positional.value, Expression::Instantiation(_))
                            || should_hug_expression(f, &positional.value, arrow_function_recursion)
                    }
                }
            } else {
                // c. The instantiation has multiple arguments and all are named.
                argument_list.arguments.iter().all(|arg| matches!(arg, Argument::Named(_))) ||
                // d. The instantiation has less than 4 non-named arguments,
                // all of which are simple expressions
                (arguments_len < 4 && argument_list.arguments.iter().all(|arg| {
                    matches!(arg, Argument::Positional(positional) if is_simple_expression(&positional.value))
                }))
            }
        }
    }
}

pub fn is_breaking_expression<'arena>(
    f: &FormatterState<'_, 'arena>,
    node: &'arena Expression<'arena>,
    arrow_function_recursion: bool,
) -> bool {
    if let Expression::Parenthesized(inner) = node {
        return is_breaking_expression(f, inner.expression, arrow_function_recursion);
    }

    if let Expression::UnaryPrefix(operation) = node {
        return is_breaking_expression(f, operation.operand, arrow_function_recursion);
    }

    if let Expression::ArrowFunction(arrow_function) = node {
        return !arrow_function_recursion && is_breaking_expression(f, arrow_function.expression, true);
    }

    if let Expression::Instantiation(Instantiation { argument_list: Some(args), .. }) = node
        && should_break_all_arguments(f, args, false)
    {
        return true;
    }

    if let Expression::Call(call) = node
        && should_break_all_arguments(f, call.get_argument_list(), false)
    {
        return true;
    }

    matches!(
        node,
        Expression::Array(_)
            | Expression::LegacyArray(_)
            | Expression::List(_)
            | Expression::Closure(_)
            | Expression::PartialApplication(_)
            | Expression::AnonymousClass(_)
            | Expression::Match(_)
    )
}

fn contains_breaking_concatenation(node: &Expression<'_>) -> bool {
    match node {
        Expression::Binary(binary) if binary.operator.is_concatenation() => {
            matches!(binary.lhs, Expression::Call(_)) || matches!(binary.rhs, Expression::Call(_))
        }
        Expression::Parenthesized(inner) => contains_breaking_concatenation(inner.expression),
        _ => false,
    }
}

pub fn is_expandable_expression<'arena>(node: &'arena Expression<'arena>, include_calls: bool) -> bool {
    if let Expression::Parenthesized(inner) = node {
        return is_expandable_expression(inner.expression, include_calls);
    }

    if let Expression::Throw(throw) = node {
        return is_expandable_expression(throw.exception, include_calls);
    }

    if let Expression::Yield(r#yield) = node {
        return match r#yield {
            Yield::Value(yield_value) => {
                if let Some(value) = yield_value.value {
                    is_expandable_expression(value, include_calls)
                } else {
                    false
                }
            }
            Yield::Pair(yield_pair) => {
                is_expandable_expression(yield_pair.key, include_calls)
                    || is_expandable_expression(yield_pair.value, include_calls)
            }
            Yield::From(yield_from) => is_expandable_expression(yield_from.iterator, include_calls),
        };
    }

    if let Expression::UnaryPrefix(operation) = node {
        return is_expandable_expression(operation.operand, include_calls);
    }

    let argument_list = match node {
        Expression::Call(call) => Some(call.get_argument_list()),
        Expression::Instantiation(instantiation) => instantiation.argument_list.as_ref(),
        _ => None,
    };

    if let Some(argument_list) = argument_list
        && argument_list.arguments.iter().any(|arg| is_expandable_expression(arg.value(), include_calls))
    {
        return true;
    }

    if let Expression::Call(_) | Expression::Instantiation(_) = node {
        return include_calls;
    }

    matches!(
        node,
        Expression::Array(_)
            | Expression::LegacyArray(_)
            | Expression::List(_)
            | Expression::Closure(_)
            | Expression::PartialApplication(_)
            | Expression::AnonymousClass(_)
            | Expression::Match(_)
    )
}

pub fn is_simple_expression<'arena>(node: &'arena Expression<'arena>) -> bool {
    if let Expression::Parenthesized(inner) = node {
        return is_simple_expression(inner.expression);
    }

    if let Expression::UnaryPrefix(operation) = node {
        return is_simple_expression(operation.operand);
    }

    if let Expression::Binary(operation) = node {
        return is_simple_expression(operation.lhs) && is_simple_expression(operation.rhs);
    }

    matches!(
        node,
        Expression::Static(_)
            | Expression::Parent(_)
            | Expression::Self_(_)
            | Expression::MagicConstant(_)
            | Expression::Literal(_)
            | Expression::Identifier(_)
            | Expression::ConstantAccess(_)
            | Expression::Variable(_)
            | Expression::Access(Access::ClassConstant(_))
    )
}

pub fn is_simple_single_line_expression<'arena>(
    f: &FormatterState<'_, 'arena>,
    node: &'arena Expression<'arena>,
) -> bool {
    if let Expression::Parenthesized(inner) = node {
        return is_simple_single_line_expression(f, inner.expression);
    }

    if let Expression::UnaryPrefix(operation) = node {
        return is_simple_single_line_expression(f, operation.operand);
    }

    if let Expression::Binary(operation) = node {
        return is_simple_single_line_expression(f, operation.lhs)
            && is_simple_single_line_expression(f, operation.rhs);
    }

    if let Expression::Literal(literal) = node {
        return match literal {
            Literal::String(literal_string) => {
                if let Some(v) = &literal_string.value
                    && string_width(v) >= f.settings.print_width
                {
                    return false;
                }

                f.file.line_number(literal_string.span.start.offset)
                    == f.file.line_number(literal_string.span.end.offset)
            }
            _ => true,
        };
    }

    if let Expression::ArrayAccess(ArrayAccess { array, index, .. }) = node {
        return is_simple_single_line_expression(f, array) && is_simple_single_line_expression(f, index);
    }

    if let Expression::Call(call) = node {
        if !call.get_argument_list().arguments.is_empty() {
            return false;
        }

        return match call {
            Call::Function(function_call) => is_simple_single_line_expression(f, function_call.function),
            Call::Method(method_call) => {
                is_simple_single_line_expression(f, method_call.object) && method_call.method.is_identifier()
            }
            Call::NullSafeMethod(method_call) => {
                is_simple_single_line_expression(f, method_call.object) && method_call.method.is_identifier()
            }
            Call::StaticMethod(method_call) => {
                is_simple_single_line_expression(f, method_call.class) && method_call.method.is_identifier()
            }
        };
    }

    matches!(
        node,
        Expression::Static(_)
            | Expression::Parent(_)
            | Expression::Self_(_)
            | Expression::MagicConstant(_)
            | Expression::Identifier(_)
            | Expression::ConstantAccess(_)
            | Expression::Variable(_)
            | Expression::Access(Access::ClassConstant(_))
    )
}

#[inline]
pub(super) const fn is_string_word_type(node: &Expression) -> bool {
    matches!(
        node,
        Expression::Static(_)
            | Expression::Parent(_)
            | Expression::Self_(_)
            | Expression::MagicConstant(_)
            | Expression::Identifier(Identifier::Local(_))
            | Expression::ConstantAccess(ConstantAccess { name: Identifier::Local(_) })
            | Expression::Variable(Variable::Direct(_))
    )
}

pub(super) fn print_colon_delimited_body<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    colon: &Span,
    statements: &'arena Sequence<'arena, Statement<'arena>>,
    end_keyword: &'arena Keyword<'arena>,
    terminator: &'arena Terminator<'arena>,
) -> Document<'arena> {
    let mut parts = vec![in f.arena;Document::String(":")];

    let mut printed_statements = print_statement_sequence(f, statements);
    if !printed_statements.is_empty() {
        if let Some(Statement::ClosingTag(_)) = statements.first() {
            printed_statements.insert(0, Document::String(" "));
            parts.push(Document::Array(printed_statements));
        } else {
            printed_statements.insert(0, Document::Line(Line::hard()));
            parts.push(Document::Indent(printed_statements));
        }
    }

    if let Some(comments) = f.print_dangling_comments(colon.join(terminator.span()), true) {
        parts.push(comments);
    } else if !matches!(statements.last(), Some(Statement::OpeningTag(_))) {
        parts.push(Document::Line(Line::hard()));
    } else {
        parts.push(Document::String(" "));
    }

    parts.push(end_keyword.format(f));
    parts.push(terminator.format(f));

    Document::Group(Group::new(parts).with_break(true))
}

pub(super) fn print_modifiers<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    modifiers: &'arena Sequence<'arena, Modifier<'arena>>,
) -> Vec<'arena, Document<'arena>> {
    let mut printed_modifiers = vec![in f.arena;];

    if let Some(modifier) = modifiers.get_final() {
        printed_modifiers.push(modifier.format(f));
    }

    if let Some(modifier) = modifiers.get_abstract() {
        printed_modifiers.push(modifier.format(f));
    }

    if let Some(modifier) = modifiers.get_first_read_visibility() {
        printed_modifiers.push(modifier.format(f));
    }

    if let Some(modifier) = modifiers.get_first_write_visibility() {
        printed_modifiers.push(modifier.format(f));
    }

    if let Some(modifier) = modifiers.get_static() {
        printed_modifiers.push(modifier.format(f));
    }

    if let Some(modifier) = modifiers.get_readonly() {
        printed_modifiers.push(modifier.format(f));
    }

    Document::join(f.arena, printed_modifiers, Separator::Space)
}

pub(super) fn print_attribute_list_sequence<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    attribute_lists: &'arena Sequence<'arena, AttributeList<'arena>>,
) -> Option<Document<'arena>> {
    if attribute_lists.is_empty() {
        return None;
    }

    let mut lists = vec![in f.arena;];
    let mut has_new_line = false;
    let mut has_potentially_long_attribute = false;
    for attribute_list in attribute_lists {
        if !has_potentially_long_attribute {
            for attribute in &attribute_list.attributes {
                has_potentially_long_attribute =
                    !attribute.argument_list.as_ref().is_none_or(|args| args.arguments.is_empty());

                if has_potentially_long_attribute {
                    break;
                }
            }
        }

        lists.push(attribute_list.format(f));

        has_new_line = has_new_line || f.is_next_line_empty(attribute_list.span());
    }

    let mut contents = vec![in f.arena;];
    let len = lists.len();
    for (i, attribute_list) in lists.into_iter().enumerate() {
        contents.push(attribute_list);

        if i != len - 1 {
            contents.push(Document::Line(Line::hard()));
        }
    }

    Some(Document::Group(Group::new(contents)))
}

pub(super) fn print_clause<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    node: &'arena Statement<'arena>,
    force_space: bool,
) -> Document<'arena> {
    let clause = node.format(f);

    adjust_clause(f, node, clause, force_space)
}

pub(super) fn adjust_clause<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    node: &'arena Statement<'arena>,
    clause: Document<'arena>,
    mut force_space: bool,
) -> Document<'arena> {
    let mut is_block = false;

    let has_trailing_segment = match f.current_node() {
        Node::IfStatementBody(b) => b.else_clause.is_some() || !b.else_if_clauses.is_empty(),
        Node::IfStatementBodyElseClause(_) => {
            if let Statement::If(_) = node {
                force_space = true;
            }

            false
        }
        Node::IfStatementBodyElseIfClause(c) => {
            if let Node::IfStatementBody(b) = f.parent_node() {
                b.else_clause.is_some()
                    || b.else_if_clauses.iter().any(|clause| clause.span().start.offset >= c.span().end.offset)
            } else {
                false
            }
        }
        Node::DoWhile(_) => true,
        _ => false,
    };

    let clause = match node {
        Statement::Noop(_) => clause,
        Statement::Block(block) => {
            is_block = true;

            let is_block_empty = block_is_empty(f, &block.left_brace, &block.right_brace);
            match f.settings.control_brace_style {
                BraceStyle::SameLine => Document::Array(vec![in f.arena;Document::space(), clause]),
                BraceStyle::NextLine => {
                    if f.settings.inline_empty_control_braces && is_block_empty {
                        Document::Array(vec![in f.arena; Document::space(), clause])
                    } else {
                        Document::Array(vec![in f.arena; Document::Line(Line::default()), clause])
                    }
                }
                BraceStyle::AlwaysNextLine => {
                    if f.settings.inline_empty_control_braces && is_block_empty {
                        Document::Array(vec![in f.arena; Document::space(), clause])
                    } else {
                        Document::Array(vec![in f.arena; Document::Line(Line::hard()), clause])
                    }
                }
            }
        }
        _ => {
            if force_space {
                Document::Array(vec![in f.arena; Document::space(), clause])
            } else {
                Document::Indent(vec![in f.arena; Document::BreakParent, Document::Line(Line::hard()), clause])
            }
        }
    };

    if has_trailing_segment {
        let is_do_while = matches!(f.current_node(), Node::DoWhile(_));

        if !is_block
            || f.is_followed_by_comment_on_next_line(node.span())
            || f.has_same_line_trailing_comment(node.span())
            || (f.settings.following_clause_on_newline && !is_do_while)
        {
            Document::Array(vec![in f.arena; clause, Document::Line(Line::hard())])
        } else {
            Document::Array(vec![in f.arena; clause, Document::space()])
        }
    } else {
        clause
    }
}

pub(super) fn print_condition<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    left_parenthesis: Span,
    condition: &'arena Expression<'arena>,
    right_parenthesis: Span,
) -> Document<'arena> {
    let was_in_condition = f.in_condition;
    f.in_condition = true;

    let has_breaking_concat = match condition {
        Expression::Call(call) => {
            call.get_argument_list().arguments.iter().any(|arg| contains_breaking_concatenation(arg.value()))
        }
        Expression::Instantiation(inst) => inst
            .argument_list
            .as_ref()
            .is_some_and(|args| args.arguments.iter().any(|arg| contains_breaking_concatenation(arg.value()))),
        _ => false,
    };

    let condition = if is_expandable_expression(condition, true)
        && !has_breaking_concat
        && !f.has_comment(condition.span(), CommentFlags::Leading | CommentFlags::Trailing)
    {
        Document::Group(Group::new(vec![
            in f.arena;
            Document::space(),
            format_token(f, left_parenthesis, "("),
            condition.format(f),
            format_token(f, right_parenthesis, ")"),
        ]))
    } else {
        let group_id = f.next_id();

        Document::Group(
            Group::new(vec![
                in f.arena;
                Document::space(),
                format_token(f, left_parenthesis, "("),
                Document::IndentIfBreak(IndentIfBreak::new(group_id, vec![
                    in f.arena;
                    Document::Line(Line::soft()),
                    condition.format(f),
                ])),
                Document::Line(Line::soft()),
                format_token(f, right_parenthesis, ")"),
            ])
            .with_id(group_id),
        )
    };

    f.in_condition = was_in_condition;

    condition
}
