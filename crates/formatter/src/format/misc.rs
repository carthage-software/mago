use fennec_ast::*;
use fennec_span::HasSpan;
use fennec_span::Span;

use crate::array;
use crate::default_line;
use crate::document::Document;
use crate::document::Line;
use crate::empty_string;
use crate::format::statement::print_statement_sequence;
use crate::format::BraceStyle;
use crate::format::Format;
use crate::group;
use crate::hardline;
use crate::if_break;
use crate::indent;
use crate::settings::StaticVisibilityOrder;
use crate::space;
use crate::static_str;
use crate::token;
use crate::Formatter;

pub(super) fn should_inline_expression<'a>(f: &mut Formatter<'a>, expression: &'a Expression) -> bool {
    if f.has_surrounding_comments(expression.span()) {
        return false;
    }

    if let Expression::Parenthesized(inner) = expression {
        return should_inline_expression(f, &inner.expression);
    }

    if let Expression::Referenced(inner) = expression {
        return should_inline_expression(f, &inner.expression);
    }

    if let Expression::Suppressed(inner) = expression {
        return should_inline_expression(f, &inner.expression);
    }

    if matches!(
        expression,
        Expression::Array(_)
            | Expression::LegacyArray(_)
            | Expression::List(_)
            | Expression::Closure(_)
            | Expression::ClosureCreation(_)
            | Expression::Call(_)
            | Expression::AnonymousClass(_)
            | Expression::Match(_)
    ) {
        return true;
    }

    return false;
}

pub(super) fn print_token_with_indented_leading_comments<'a>(
    f: &mut Formatter<'a>,
    span: Span,
    value: &'a str,
    mut should_break: bool,
) -> Document<'a> {
    let mut parts = vec![];
    if let Some(leading_comments) = f.print_leading_comments(span, false) {
        parts.push(indent!(Document::Line(Line::hardline()), leading_comments));

        should_break = true;
    }

    if should_break {
        parts.push(Document::Line(Line::hardline()));
        parts.push(Document::BreakParent);
    }

    parts.push(static_str!(value));
    if let Some(trailing_comments) = f.print_trailing_comments(span, true) {
        parts.push(trailing_comments);
    }

    group!(@parts)
}

pub(super) fn print_colon_delimited_body<'a>(
    f: &mut Formatter<'a>,
    colon: &'a Span,
    statements: &'a Sequence<Statement>,
    end_keyword: &'a Keyword,
    terminator: &'a Terminator,
) -> Document<'a> {
    let mut parts = vec![token!(f, *colon, ":")];

    let statements = print_statement_sequence(f, &statements);
    let has_statements = !statements.is_empty();
    if has_statements {
        parts.push(indent!(@hardline!()));
    }

    for stmt in statements {
        parts.push(indent!(stmt));
    }

    parts.extend(hardline!());
    parts.push(end_keyword.format(f));
    parts.push(terminator.format(f));

    group!(@parts)
}

pub(super) fn print_modifiers<'a>(f: &mut Formatter<'a>, modifiers: &'a Sequence<Modifier>) -> Document<'a> {
    let mut parts = vec![];

    if let Some(modifier) = modifiers.get_final() {
        parts.push(modifier.format(f));
        parts.push(space!());
    }

    if let Some(modifier) = modifiers.get_abstract() {
        parts.push(modifier.format(f));
        parts.push(space!());
    }

    match f.settings.static_visibility_order {
        StaticVisibilityOrder::VisibilityFirst => {
            if let Some(modifier) = modifiers.get_first_visibility() {
                parts.push(modifier.format(f));
                parts.push(space!());
            }

            if let Some(modifier) = modifiers.get_static() {
                parts.push(modifier.format(f));
                parts.push(space!());
            } else if let Some(modifier) = modifiers.get_readonly() {
                parts.push(modifier.format(f));
                parts.push(space!());
            }
        }
        StaticVisibilityOrder::StaticFirst => {
            if let Some(modifier) = modifiers.get_static() {
                parts.push(modifier.format(f));
                parts.push(space!());
            } else if let Some(modifier) = modifiers.get_readonly() {
                parts.push(modifier.format(f));
                parts.push(space!());
            }

            if let Some(modifier) = modifiers.get_first_visibility() {
                parts.push(modifier.format(f));
                parts.push(space!());
            }
        }
    }

    group!(@parts)
}

pub(super) fn print_attribute_list_sequence<'a>(
    f: &mut Formatter<'a>,
    attribute_lists: &'a Sequence<AttributeList>,
) -> Option<Document<'a>> {
    if attribute_lists.is_empty() {
        return None;
    }

    let mut lists = vec![];
    let mut has_new_line = false;
    for attribute_list in attribute_lists.iter() {
        lists.push(attribute_list.format(f));

        has_new_line = f.is_next_line_empty(attribute_list.span());
    }

    // if there is a single attribute list, we can inline it
    if !has_new_line && f.settings.inline_single_attribute_group && lists.len() == 1 {
        let attribute_list = lists.remove(0);

        let mut parts = vec![];
        parts.push(attribute_list);
        parts.push(if_break!(array!(default_line!(), Document::BreakParent), space!()));

        return Some(group!(@parts));
    }

    let mut parts = vec![];
    for attribute_list in lists {
        parts.push(attribute_list);
        parts.extend(hardline!());
    }

    Some(group!(@parts))
}

pub(super) fn print_clause<'a>(f: &mut Formatter<'a>, node: &'a Statement, force_space: bool) -> Document<'a> {
    let clause = node.format(f);
    let clause = adjust_clause(f, &node, clause, force_space);

    clause
}

pub(super) fn adjust_clause<'a>(
    f: &mut Formatter<'a>,
    node: &'a Statement,
    clause: Document<'a>,
    mut force_space: bool,
) -> Document<'a> {
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
        Statement::Noop(s) => token!(f, *s, ";"),
        Statement::Block(_) => {
            is_block = true;

            match f.settings.control_brace_style {
                BraceStyle::SameLine => array!(space!(), clause),
                BraceStyle::NextLine => array!(default_line!(), clause),
            }
        }
        _ => {
            if force_space {
                array!(space!(), clause)
            } else {
                indent!(array!(@hardline!()), clause)
            }
        }
    };

    if has_trailing_segment {
        if is_block {
            return array!(clause, space!());
        } else {
            return array!(clause, array!(@hardline!()));
        }
    } else {
        clause
    }
}

pub(super) fn print_condition<'a>(
    f: &mut Formatter<'a>,
    left_parenthesis: Span,
    condition: &'a Expression,
    right_parenthesis: Span,
) -> Document<'a> {
    group!(
        token!(f, left_parenthesis, "("),
        if f.settings.control_space_parens { space!() } else { empty_string!() },
        condition.format(f),
        if f.settings.control_space_parens { space!() } else { empty_string!() },
        token!(f, right_parenthesis, ")")
    )
}
