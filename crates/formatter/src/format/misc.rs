use mago_ast::*;
use mago_span::HasSpan;
use mago_span::Span;

use crate::Formatter;
use crate::comment::CommentFlags;
use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::document::Separator;
use crate::format::Format;
use crate::format::call::collect_member_access_chain;
use crate::format::statement::print_statement_sequence;
use crate::settings::BraceStyle;

pub(super) fn has_new_line_in_range(text: &str, start: usize, end: usize) -> bool {
    text[start..end].contains('\n')
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
pub(super) fn should_hug_expression<'a>(f: &Formatter<'a>, expression: &'a Expression) -> bool {
    if let Expression::Parenthesized(inner) = expression {
        return should_hug_expression(f, &inner.expression);
    }

    if let Expression::UnaryPrefix(operation) = expression {
        return should_hug_expression(f, &operation.operand);
    }

    // if the expression has leading or trailing comments, we can't hug it
    if f.has_comment(expression.span(), CommentFlags::Leading | CommentFlags::Trailing) {
        return false;
    }

    if let Expression::Call(_) = expression {
        // Don't hug calls if they are part of a member access chain
        return collect_member_access_chain(expression)
            .is_none_or(|chain| chain.accesses.len() < f.settings.method_chain_break_threshold);
    }

    matches!(
        expression,
        Expression::Array(_)
            | Expression::LegacyArray(_)
            | Expression::List(_)
            | Expression::Closure(_)
            | Expression::ClosureCreation(_)
            | Expression::AnonymousClass(_)
            | Expression::Match(_)
            | Expression::Call(_)
    )
}

pub(super) fn is_string_word_type(node: &Expression) -> bool {
    match node {
        Expression::Static(_) | Expression::Parent(_) | Expression::Self_(_) => true,
        Expression::MagicConstant(_) => true,
        Expression::Identifier(identifier) => matches!(identifier, Identifier::Local(_)),
        Expression::Variable(variable) => {
            matches!(variable, Variable::Direct(_))
        }
        _ => false,
    }
}

pub(super) fn print_colon_delimited_body<'a>(
    f: &mut Formatter<'a>,
    colon: &'a Span,
    statements: &'a Sequence<Statement>,
    end_keyword: &'a Keyword,
    terminator: &'a Terminator,
) -> Document<'a> {
    let mut parts = vec![Document::String(":")];

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

pub(super) fn print_modifiers<'a>(f: &mut Formatter<'a>, modifiers: &'a Sequence<Modifier>) -> Vec<Document<'a>> {
    let mut printed_modifiers = vec![];

    if let Some(modifier) = modifiers.get_final() {
        printed_modifiers.push(modifier.format(f));
    }

    if let Some(modifier) = modifiers.get_abstract() {
        printed_modifiers.push(modifier.format(f));
    }

    if f.settings.static_before_visibility {
        if let Some(modifier) = modifiers.get_static() {
            printed_modifiers.push(modifier.format(f));
        }

        if let Some(modifier) = modifiers.get_readonly() {
            printed_modifiers.push(modifier.format(f));
        }

        if let Some(modifier) = modifiers.get_first_read_visibility() {
            printed_modifiers.push(modifier.format(f));
        }

        if let Some(modifier) = modifiers.get_first_write_visibility() {
            printed_modifiers.push(modifier.format(f));
        }
    } else {
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
    }

    Document::join(printed_modifiers, Separator::Space)
}

pub(super) fn print_attribute_list_sequence<'a>(
    f: &mut Formatter<'a>,
    attribute_lists: &'a Sequence<AttributeList>,
    can_inline: bool,
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
    if can_inline && !has_new_line && lists.len() == 1 {
        return Some(Document::Group(Group::new(vec![lists.remove(0), Document::Line(Line::default())])));
    }

    let mut contents = vec![];
    for attribute_list in lists {
        contents.push(attribute_list);
        contents.push(Document::Line(Line::hard()));
    }

    Some(Document::Group(Group::new(contents).with_break(true)))
}

pub(super) fn print_clause<'a>(f: &mut Formatter<'a>, node: &'a Statement, force_space: bool) -> Document<'a> {
    let clause = node.format(f);
    let clause = adjust_clause(f, node, clause, force_space);

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
        Statement::Noop(_) => clause,
        Statement::Block(_) => {
            is_block = true;

            match f.settings.control_brace_style {
                BraceStyle::SameLine => Document::Array(vec![Document::space(), clause]),
                BraceStyle::NextLine => Document::Array(vec![Document::Line(Line::default()), clause]),
            }
        }
        _ => {
            if force_space {
                Document::Array(vec![Document::space(), clause])
            } else {
                Document::Indent(vec![Document::BreakParent, Document::Line(Line::hard()), clause])
            }
        }
    };

    if has_trailing_segment {
        if is_block {
            Document::Array(vec![clause, Document::space()])
        } else {
            Document::Indent(vec![Document::BreakParent, clause, Document::Line(Line::hard())])
        }
    } else {
        clause
    }
}

pub(super) fn print_condition<'a>(f: &mut Formatter<'a>, condition: &'a Expression) -> Document<'a> {
    Document::Group(Group::new(vec![
        Document::String("("),
        Document::Indent(vec![
            Document::Line(if f.settings.control_space_parens { Line::default() } else { Line::soft() }),
            condition.format(f),
        ]),
        Document::Line(if f.settings.control_space_parens { Line::default() } else { Line::soft() }),
        Document::String(")"),
    ]))
}
