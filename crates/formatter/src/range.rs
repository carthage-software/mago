use std::ops::Range;

use mago_allocator::Arena;
use mago_database::file::File;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Statement;
use mago_syntax::error::ParseError;
use mago_syntax::parser::parse_file_with_settings;

use crate::Formatter;
use crate::has_format_ignore_comment;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::printer::Printer;

/// A formatted statement, member, or import group and the source it replaces.
#[derive(Debug)]
pub struct FormattedRange<'arena> {
    pub range: Range<u32>,
    pub new_text: &'arena [u8],
}

impl<'arena, A: Arena> Formatter<'arena, A> {
    /// Format the smallest statement or member enclosing a byte range.
    ///
    /// Imports expand to their full group. Comments and leading indentation
    /// belong to the replacement; bytes outside its returned range stay intact.
    /// Empty ranges select the statement at the cursor. Invalid ranges, HTML-only
    /// selections, ignored regions, and unchanged text return `None`.
    ///
    /// # Errors
    ///
    /// Returns a parse error if the source or the resulting file is invalid.
    pub fn format_range(
        &self,
        file: &File,
        mut range: Range<u32>,
    ) -> Result<Option<FormattedRange<'arena>>, ParseError> {
        if range.start > range.end {
            return Ok(None);
        }

        if range.end > file.size
            || range.is_empty() && file.contents.get(range.start as usize).is_none_or(u8::is_ascii_whitespace)
        {
            return Ok(None);
        }

        if !range.is_empty() {
            while range.start < range.end && file.contents[range.start as usize].is_ascii_whitespace() {
                range.start += 1;
            }

            while range.start < range.end && file.contents[range.end as usize - 1].is_ascii_whitespace() {
                range.end -= 1;
            }

            if range.is_empty() {
                return Ok(None);
            }
        }

        let program = parse_file_with_settings(self.arena, file, self.parser_settings);
        if let Some(error) = program.errors.first() {
            return Err(error.clone());
        }

        if has_format_ignore_comment(program) {
            return Ok(None);
        }

        let root = Node::Program(program);
        let mut selected = (root.span(), NodeKind::Program);
        if select(root, &range, &mut selected) {
            return Ok(None);
        }

        let (mut source_start, source_end, output, mut start, end) = if selected.1 == NodeKind::Program {
            let output = self.format(file, program);
            (0, file.size, output, 0, output.len())
        } else {
            let mut state = FormatterState::new(self.arena, program, file, self.php_version, self.settings);
            state.set_range(selected.0, selected.1);
            let document = program.format(&mut state);
            let Some((output, printed)) =
                Printer::new(self.arena, document, file.size as usize, self.settings).build_range()
            else {
                return Ok(None);
            };

            let Some((source_start, source_end)) = state.range_source() else {
                return Ok(None);
            };

            (source_start, source_end, output, printed.start, printed.end)
        };

        let source_line = line_start(&file.contents, source_start as usize);
        if file.contents[source_line..source_start as usize].iter().all(|b| matches!(b, b' ' | b'\t')) {
            let output_line = line_start(output, start);
            if output[output_line..start].iter().all(|b| matches!(b, b' ' | b'\t')) {
                source_start = source_line as u32;
                start = output_line;
            }
        }

        let new_text = &output[start..end];
        if new_text == &file.contents[source_start as usize..source_end as usize] {
            return Ok(None);
        }

        let mut updated = file.contents[..source_start as usize].to_vec();
        updated.extend_from_slice(new_text);
        updated.extend_from_slice(&file.contents[source_end as usize..]);
        let updated = File::ephemeral(b"range.php".as_slice().into(), updated.into());
        let parsed = parse_file_with_settings(self.arena, &updated, self.parser_settings);
        if let Some(error) = parsed.errors.first() {
            return Err(error.clone());
        }

        Ok(Some(FormattedRange { range: source_start..source_end, new_text }))
    }
}

fn line_start(source: &[u8], offset: usize) -> usize {
    source[..offset].iter().rposition(|b| matches!(b, b'\r' | b'\n')).map_or(0, |i| i + 1)
}

fn contains(span: Span, range: &Range<u32>) -> bool {
    span.start.offset <= range.start
        && range.end <= span.end.offset
        && (!range.is_empty() || range.start < span.end.offset)
}

fn select(node: Node<'_, '_>, range: &Range<u32>, selected: &mut (Span, NodeKind)) -> bool {
    if !contains(node.span(), range) {
        return false;
    }

    if matches!(node, Node::Inline(_)) {
        return true;
    }

    let eligible = matches!(node, Node::ClassLikeMember(_))
        || matches!(node, Node::Statement(statement) if !matches!(statement,
            Statement::OpeningTag(_) | Statement::ClosingTag(_) | Statement::Inline(_)
                | Statement::Use(_) | Statement::HaltCompiler(_)));
    if eligible && node.span().length() < selected.0.length() {
        *selected = (node.span(), node.kind());
    }

    let children = node.children();
    let is_use = |node: &Node<'_, '_>| matches!(node, Node::Statement(Statement::Use(_)));
    for group in children.chunk_by(|a, b| is_use(a) && is_use(b)) {
        if is_use(&group[0]) {
            let span = group[0].span().join(group[group.len() - 1].span());
            if contains(span, range) && span.length() < selected.0.length() {
                *selected = (span, NodeKind::Use);
            }
        }
    }

    children.into_iter().any(|child| select(child, range, selected))
}
