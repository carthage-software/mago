use fennec_ast::*;
use fennec_span::HasSpan;
use fennec_span::Span;

use crate::array;
use crate::comment::CommentFlags;
use crate::document::Document;
use crate::document::Line;
use crate::format::statement::print_statement_sequence;
use crate::format::BraceStyle;
use crate::format::Format;
use crate::group;
use crate::indent;
use crate::settings::StaticVisibilityOrder;
use crate::static_str;
use crate::token;
use crate::Formatter;

use super::Group;

pub(super) fn has_new_line_in_range<'a>(text: &'a str, start: usize, end: usize) -> bool {
    text[(start as usize)..(end as usize)].contains('\n')
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

    matches!(
        expression,
        Expression::Array(_)
            | Expression::LegacyArray(_)
            | Expression::List(_)
            | Expression::Closure(_)
            | Expression::ClosureCreation(_)
            | Expression::Call(_)
            | Expression::AnonymousClass(_)
            | Expression::Match(_)
    )
}

pub(super) fn is_string_word_type<'a>(node: &'a Expression) -> bool {
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

pub(super) fn print_token_with_indented_leading_comments<'a>(
    f: &mut Formatter<'a>,
    span: Span,
    value: &'a str,
    mut should_break: bool,
) -> Document<'a> {
    let mut parts = vec![];
    if let Some(leading_comments) = f.print_leading_comments(span) {
        parts.push(indent!(Document::Line(Line::hardline()), leading_comments));

        should_break = true;
    }

    if should_break {
        parts.push(Document::Line(Line::hardline()));
        parts.push(Document::BreakParent);
    }

    parts.push(static_str!(value));
    if let Some(trailing_comments) = f.print_trailing_comments(span) {
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

    let mut statements = print_statement_sequence(f, &statements);
    if !statements.is_empty() {
        statements.insert(0, Document::Line(Line::hardline()));

        parts.push(Document::Indent(statements));
    }

    if let Some(comments) = f.print_dangling_comments(colon.join(terminator.span()), true) {
        parts.push(comments);
    } else {
        parts.push(Document::Line(Line::hardline()));
    }

    parts.push(end_keyword.format(f));
    parts.push(terminator.format(f));

    Document::Group(Group::new(parts).with_break(true))
}

pub(super) fn print_modifiers<'a>(f: &mut Formatter<'a>, modifiers: &'a Sequence<Modifier>) -> Vec<Document<'a>> {
    let mut printed_modifiers = vec![];

    if let Some(modifier) = modifiers.get_final() {
        printed_modifiers.push(modifier.format(f));
        printed_modifiers.push(Document::space());
    }

    if let Some(modifier) = modifiers.get_abstract() {
        printed_modifiers.push(modifier.format(f));
        printed_modifiers.push(Document::space());
    }

    match f.settings.static_visibility_order {
        StaticVisibilityOrder::VisibilityFirst => {
            if let Some(modifier) = modifiers.get_first_visibility() {
                printed_modifiers.push(modifier.format(f));
                printed_modifiers.push(Document::space());
            }

            if let Some(modifier) = modifiers.get_static() {
                printed_modifiers.push(modifier.format(f));
                printed_modifiers.push(Document::space());
            } else if let Some(modifier) = modifiers.get_readonly() {
                printed_modifiers.push(modifier.format(f));
                printed_modifiers.push(Document::space());
            }
        }
        StaticVisibilityOrder::StaticFirst => {
            if let Some(modifier) = modifiers.get_static() {
                printed_modifiers.push(modifier.format(f));
                printed_modifiers.push(Document::space());
            } else if let Some(modifier) = modifiers.get_readonly() {
                printed_modifiers.push(modifier.format(f));
                printed_modifiers.push(Document::space());
            }

            if let Some(modifier) = modifiers.get_first_visibility() {
                printed_modifiers.push(modifier.format(f));
                printed_modifiers.push(Document::space());
            }
        }
    }

    printed_modifiers
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
        parts.push(Document::Line(Line::default()));

        return Some(group!(@parts));
    }

    let mut parts = vec![];
    for attribute_list in lists {
        parts.push(attribute_list);
        parts.push(Document::Line(Line::default()));
    }

    Some(Document::Group(Group::new(parts).with_break(true)))
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
        Statement::Noop(_) => clause,
        Statement::Block(_) => {
            is_block = true;

            match f.settings.control_brace_style {
                BraceStyle::SameLine => array!(Document::space(), clause),
                BraceStyle::NextLine => array!(Document::Line(Line::default()), clause),
            }
        }
        _ => {
            if force_space {
                Document::Array(vec![Document::space(), clause])
            } else {
                Document::Indent(vec![Document::BreakParent, Document::Line(Line::hardline()), clause])
            }
        }
    };

    if has_trailing_segment {
        if is_block {
            Document::Array(vec![clause, Document::space()])
        } else {
            Document::Indent(vec![Document::BreakParent, clause, Document::Line(Line::hardline())])
        }
    } else {
        clause
    }
}

pub(super) fn print_condition<'a>(f: &mut Formatter<'a>, condition: &'a Expression) -> Document<'a> {
    Document::Group(Group::new(vec![
        Document::String("("),
        if f.settings.control_space_parens { Document::space() } else { Document::empty() },
        condition.format(f),
        if f.settings.control_space_parens { Document::space() } else { Document::empty() },
        Document::String(")"),
    ]))
}
