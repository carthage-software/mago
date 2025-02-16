use mago_ast::*;
use mago_span::HasSpan;

use crate::document::Document;
use crate::document::Line;
use crate::format::Format;
use crate::Formatter;

pub fn print_statement_sequence<'a, 'alloc>(
    f: &mut Formatter<'a, 'alloc>,
    stmts: &'a Sequence<'alloc, Statement<'alloc>>,
) -> Vec<Document<'a>> {
    let mut parts = vec![];

    let last_non_noop_index = stmts.iter().rposition(|stmt| !matches!(stmt, Statement::Noop(_)));
    for (i, stmt) in stmts.iter().enumerate() {
        let mut should_add_space = false;

        let should_add_new_line = match stmt {
            Statement::ClosingTag(_) => false,
            Statement::Inline(_) => false,
            Statement::Expression(ExpressionStatement { terminator: Terminator::ClosingTag(_), .. }) => false,
            Statement::OpeningTag(_) => {
                if let Some(index) = f.skip_to_line_end(Some(stmt.span().end_position().offset)) {
                    should_add_space = !f.has_newline(index, false);
                }

                true
            }
            _ => {
                if f.has_newline(stmt.span().end_position().offset, false) {
                    true
                } else if let Some(Statement::ClosingTag(tag)) = stmts.get(i + 1) {
                    if f.skip_spaces_and_new_lines(Some(tag.span.end.offset), false).is_some() {
                        should_add_space = true;
                    }

                    false
                } else {
                    true
                }
            }
        };

        parts.push(stmt.format(f));

        let is_last = if let Some(index) = last_non_noop_index { i == index } else { i == stmts.len() - 1 };

        if should_add_space {
            if !is_last {
                parts.push(Document::space());
            }
        } else if should_add_new_line {
            if let Some(index) = last_non_noop_index {
                if i != index {
                    parts.push(Document::Line(Line::hardline()));
                    if f.is_next_line_empty(stmt.span()) {
                        parts.push(Document::Line(Line::hardline()));
                    }
                }
            }
        }
    }

    parts
}
