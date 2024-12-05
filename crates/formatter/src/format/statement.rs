use fennec_ast::*;
use fennec_span::HasSpan;

use crate::document::Document;
use crate::document::Line;
use crate::format::Format;
use crate::Formatter;

pub fn print_statement_sequence<'a>(f: &mut Formatter<'a>, stmts: &'a Sequence<Statement>) -> Vec<Document<'a>> {
    let mut parts = vec![];

    let mut should_include_new_line = true;
    let last_non_noop_index = stmts.iter().rposition(|stmt| !matches!(stmt, Statement::Noop(_)));
    for (i, stmt) in stmts.iter().enumerate() {
        if matches!(stmt, Statement::ClosingTag(_)) {
            // stop including new lines after closing tags
            should_include_new_line = false;
        }

        if matches!(stmt, Statement::OpeningTag(_)) {
            // start including new lines after opening tags
            should_include_new_line = true;
        }

        parts.push(stmt.format(f));

        if should_include_new_line {
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
