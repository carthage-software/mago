use mago_ast::*;
use mago_span::HasSpan;
use mago_span::Span;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::format::statement;
use crate::format::Format;
use crate::Formatter;

pub(super) fn print_block_of_nodes<'a, 'alloc, T: Format<'a, 'alloc> + HasSpan>(
    f: &mut Formatter<'a, 'alloc>,
    left_brace: &'a Span,
    nodes: &'a Sequence<'alloc, T>,
    right_brace: &'a Span,
    inline_empty: bool,
) -> Document<'a> {
    let length = nodes.len();
    let mut contents = vec![Document::String("{"), {
        if length == 0 {
            Document::empty()
        } else {
            let mut formatted = vec![Document::Line(Line::hardline())];
            for (i, item) in nodes.iter().enumerate() {
                formatted.push(item.format(f));

                if i < (length - 1) {
                    formatted.push(Document::Line(Line::hardline()));
                    if f.is_next_line_empty(item.span()) {
                        formatted.push(Document::Line(Line::hardline()));
                    }
                }
            }

            Document::Indent(formatted)
        }
    }];

    if let Some(comments) = f.print_dangling_comments(left_brace.join(*right_brace), true) {
        contents.push(comments);
    } else if length > 0 || !inline_empty {
        contents.push(Document::Line(Line::hardline()));
    }

    contents.push(Document::String("}"));

    Document::Group(Group::new(contents))
}

pub(super) fn print_block<'a, 'alloc>(
    f: &mut Formatter<'a, 'alloc>,
    left_brace: &Span,
    stmts: &'a Sequence<'alloc, Statement<'alloc>>,
    right_brace: &Span,
) -> Document<'a> {
    let mut contents = vec![];
    contents.push(Document::String("{"));
    let has_body = stmts.iter().any(|stmt| !matches!(stmt, Statement::Noop(_)));
    let should_break = if has_body {
        let mut statements = statement::print_statement_sequence(f, stmts);
        statements.insert(0, Document::Line(Line::hardline()));
        contents.push(Document::Indent(statements));
        true
    } else {
        let parent = f.parent_node();
        // in case the block is empty, we still want to add a new line
        // in some cases.
        match &parent {
            // functions, closures, and methods
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
    } else {
        contents.push(Document::Line(Line::softline()));
    }

    contents.push(Document::String("}"));

    Document::Group(Group::new(contents).with_break(should_break))
}

pub(super) fn print_block_body<'a, 'alloc>(
    f: &mut Formatter<'a, 'alloc>,
    stmts: &'a Sequence<'alloc, Statement<'alloc>>,
) -> Option<Document<'a>> {
    let has_body = stmts.iter().any(|stmt| !matches!(stmt, Statement::Noop(_)));

    if has_body {
        Some(Document::Array(statement::print_statement_sequence(f, stmts)))
    } else {
        None
    }
}
