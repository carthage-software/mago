use mago_ast::*;
use mago_span::HasSpan;
use mago_span::Span;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::format::statement;

pub(super) fn print_block_of_nodes<'a, T: Format<'a> + HasSpan>(
    f: &mut FormatterState<'a>,
    left_brace: &Span,
    nodes: &'a Sequence<T>,
    right_brace: &Span,
    inline_empty: bool,
) -> Document<'a> {
    let length = nodes.len();
    let mut contents = vec![Document::String("{"), {
        if length == 0 {
            Document::empty()
        } else {
            let mut formatted = vec![Document::Line(Line::hard())];
            for (i, item) in nodes.iter().enumerate() {
                formatted.push(item.format(f));

                if i < (length - 1) {
                    formatted.push(Document::Line(Line::hard()));
                    if f.is_next_line_empty(item.span()) {
                        formatted.push(Document::Line(Line::hard()));
                    }
                }
            }

            Document::Indent(formatted)
        }
    }];

    if let Some(comments) = f.print_dangling_comments(left_brace.join(*right_brace), true) {
        contents.push(comments);
    } else if length > 0 || !inline_empty {
        contents.push(Document::Line(Line::hard()));
    }

    contents.push(Document::String("}"));

    Document::Group(Group::new(contents))
}

pub(super) fn print_block<'a>(
    f: &mut FormatterState<'a>,
    left_brace: &Span,
    stmts: &'a Sequence<Statement>,
    right_brace: &Span,
) -> Document<'a> {
    let mut contents = vec![];
    contents.push(Document::String("{"));
    let has_body = stmts.iter().any(|stmt| !matches!(stmt, Statement::Noop(_)));
    let has_inline_body = has_body && {
        matches!((stmts.first(), stmts.last()), (Some(Statement::ClosingTag(_)), Some(Statement::OpeningTag(_))))
    };

    let should_break = if has_body {
        let mut statements = statement::print_statement_sequence(f, stmts);
        if has_inline_body {
            statements.insert(0, Document::space());
        } else {
            statements.insert(0, Document::Line(Line::hard()));
        }

        contents.push(Document::Indent(statements));

        true
    } else {
        let parent = f.parent_node();
        // in case the block is empty, we still want to add a new line
        // in some cases.
        match &parent {
            // functions, and methods
            Node::Function(_) | Node::MethodBody(_) | Node::PropertyHookConcreteBody(_) => true,
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
                    ) => true,
                    _ => false,
                }
            }
            _ => false,
        }
    };

    if let Some(comments) = f.print_dangling_comments(left_brace.join(*right_brace), true) {
        contents.push(comments);
    } else if has_inline_body {
        contents.push(Document::space());
    } else if should_break {
        contents.push(Document::Line(Line::soft()));
    }

    contents.push(Document::String("}"));

    Document::Group(Group::new(contents).with_break(should_break))
}

pub(super) fn print_block_body<'a>(f: &mut FormatterState<'a>, stmts: &'a Sequence<Statement>) -> Option<Document<'a>> {
    let has_body = stmts.iter().any(|stmt| !matches!(stmt, Statement::Noop(_)));

    if has_body { Some(Document::Array(statement::print_statement_sequence(f, stmts))) } else { None }
}
