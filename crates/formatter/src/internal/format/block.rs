use bumpalo::vec;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::format::statement;

pub(super) fn print_block_of_nodes<'ast, 'arena, T: Format<'arena> + HasSpan>(
    f: &mut FormatterState<'_, 'arena>,
    left_brace: &Span,
    nodes: &'arena Sequence<'arena, T>,
    right_brace: &Span,
    inline_empty: bool,
) -> Document<'arena> {
    let length = nodes.len();
    let mut contents = vec![in f.arena; Document::String("{")];
    if let Some(c) = f.print_trailing_comments(*left_brace) {
        contents.push(c);
    }

    if length != 0 {
        let mut formatted = vec![in f.arena; Document::Line(Line::hard())];
        for (i, item) in nodes.iter().enumerate() {
            formatted.push(item.format(f));

            if i < (length - 1) {
                formatted.push(Document::Line(Line::hard()));
                if f.is_next_line_empty(item.span()) {
                    formatted.push(Document::Line(Line::hard()));
                }
            }
        }

        contents.push(Document::Indent(formatted));
    }

    if let Some(comments) = f.print_dangling_comments(left_brace.join(*right_brace), true) {
        if length > 0 && f.settings.empty_line_before_dangling_comments {
            contents.push(Document::Line(Line::soft()));
        }

        contents.push(comments);
    } else if length > 0 || !inline_empty {
        contents.push(Document::Line(Line::hard()));
    }

    contents.push(Document::String("}"));
    if let Some(comments) = f.print_trailing_comments(*right_brace) {
        contents.push(comments);
    }

    Document::Group(Group::new(contents))
}

pub(super) fn print_block<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    left_brace: &'arena Span,
    stmts: &'arena Sequence<'arena, Statement<'arena>>,
    right_brace: &'arena Span,
) -> Document<'arena> {
    let mut contents = vec![in f.arena];
    contents.push(Document::String("{"));
    if let Some(c) = f.print_trailing_comments(*left_brace) {
        contents.push(c);
    }

    let has_statements = stmts.iter().any(|stmt| !matches!(stmt, Statement::Noop(_)));
    let is_empty = !has_statements && block_is_empty(f, left_brace, right_brace);

    let has_inline_body = has_statements && {
        matches!((stmts.first(), stmts.last()), (Some(Statement::ClosingTag(_)), Some(Statement::OpeningTag(_))))
    };

    let should_break = if has_statements {
        let mut statements = statement::print_statement_sequence(f, stmts);
        if has_inline_body {
            statements.insert(0, Document::space());
        } else {
            statements.insert(0, Document::Line(Line::hard()));
        }

        contents.push(Document::Indent(statements));

        true
    } else if !is_empty {
        true
    } else {
        let parent = f.parent_node();
        // in case the block is empty, we still want to add a new line
        // in some cases.
        match &parent {
            // functions, and methods
            Node::MethodBody(_) => {
                if let Some(Node::Method(method)) = f.grandparent_node() {
                    if method.name.value.eq_ignore_ascii_case("__construct") {
                        !f.settings.inline_empty_constructor_braces
                    } else {
                        !f.settings.inline_empty_method_braces
                    }
                } else {
                    !f.settings.inline_empty_method_braces
                }
            }
            Node::Closure(_) => !f.settings.inline_empty_closure_braces,
            Node::PropertyHookConcreteBody(_) => !f.settings.inline_empty_method_braces,
            Node::Function(_) => !f.settings.inline_empty_function_braces,
            // try, catch, finally
            Node::Try(_) | Node::TryCatchClause(_) | Node::TryFinallyClause(_) => true,
            Node::Statement(_) => {
                let grand_parent = f.grandparent_node();

                match grand_parent {
                    // control structures
                    Some(
                        Node::ForBody(_)
                        | Node::WhileBody(_)
                        | Node::DoWhile(_)
                        | Node::If(_)
                        | Node::IfStatementBody(_)
                        | Node::IfStatementBodyElseClause(_)
                        | Node::IfStatementBodyElseIfClause(_)
                        | Node::ForeachBody(_),
                    ) => !f.settings.inline_empty_control_braces,
                    _ => false,
                }
            }
            _ => false,
        }
    };

    if let Some(comments) = f.print_dangling_comments(left_brace.join(*right_brace), true) {
        if has_statements && f.settings.empty_line_before_dangling_comments {
            contents.push(Document::Line(Line::soft()));
        }

        contents.push(comments);
    } else if has_inline_body {
        contents.push(Document::space());
    } else if should_break {
        contents.push(Document::Line(Line::soft()));
    }

    contents.push(Document::String("}"));
    if let Some(comments) = f.print_trailing_comments(*right_brace) {
        contents.push(comments);
    }

    Document::Group(Group::new(contents).with_break(should_break))
}

pub(super) fn print_block_body<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    stmts: &'arena Sequence<'arena, Statement<'arena>>,
) -> Option<Document<'arena>> {
    let has_body = stmts.iter().any(|stmt| !matches!(stmt, Statement::Noop(_)));

    if has_body { Some(Document::Array(statement::print_statement_sequence(f, stmts))) } else { None }
}

pub fn block_is_empty(f: &mut FormatterState<'_, '_>, left_brace: &Span, right_brace: &Span) -> bool {
    let content = &f.source_text[left_brace.end.offset as usize..right_brace.start.offset as usize];

    for c in content.chars() {
        if !c.is_whitespace() || matches!(c, ';') {
            return false;
        }
    }

    true
}
