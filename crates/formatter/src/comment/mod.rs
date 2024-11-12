use fennec_ast::Trivia;
use fennec_ast::TriviaKind;
use fennec_span::Span;

use crate::array;
use crate::document::Document;
use crate::document::Line;
use crate::space;
use crate::static_str;
use crate::Formatter;

impl<'a> Formatter<'a> {
    #[must_use]
    pub(crate) fn print_comments(
        &mut self,
        before: Option<Document<'a>>,
        doc: Document<'a>,
        after: Option<Document<'a>>,
    ) -> Document<'a> {
        match (before, after) {
            (Some(before), Some(after)) => array!(before, doc, after),
            (Some(before), None) => array!(before, doc),
            (None, Some(after)) => array!(doc, after),
            (None, None) => doc,
        }
    }

    #[must_use]
    pub(crate) fn has_surrounding_comments(&self, span: Span) -> bool {
        self.has_trailing_comments(span) || self.has_leading_comments(span)
    }

    #[must_use]
    pub(crate) fn has_leading_multi_line_comments(&self, span: Span) -> bool {
        self.trivias
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if !self.used_trivia_indices.contains(&i) && t.span.end.offset <= span.start.offset {
                    Some(t)
                } else {
                    None
                }
            })
            .any(|t| {
                if t.kind.is_block_comment() {
                    self.source.line_number(t.span.start.offset) < self.source.line_number(span.start.offset)
                } else {
                    false
                }
            })
    }

    #[must_use]
    pub(crate) fn has_leading_comments(&self, span: Span) -> bool {
        self.trivias
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if !self.used_trivia_indices.contains(&i) && t.span.end.offset <= span.start.offset {
                    Some(t)
                } else {
                    None
                }
            })
            .any(|t| t.kind.is_comment())
    }

    #[must_use]
    pub(crate) fn print_leading_comments(&mut self, span: Span, trailing_new_line: bool) -> Option<Document<'a>> {
        // collect all trivias that are before the span, and have not been used yet
        let trivias: Vec<_> = self
            .trivias
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if !self.used_trivia_indices.contains(&i) && t.span.end.offset <= span.start.offset {
                    self.used_trivia_indices.insert(i);

                    Some(t)
                } else {
                    None
                }
            })
            .copied()
            .collect();

        self.write_trivias(&trivias, false, trailing_new_line)
    }

    #[must_use]
    pub(crate) fn has_trailing_single_line_comments(&self, span: Span) -> bool {
        let ending_line = self.source.line_number(span.end.offset);
        self.trivias
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if span.end.offset <= t.span.start.offset
                    && !self.used_trivia_indices.contains(&i)
                    && self.source.line_number(t.span.start.offset) == ending_line
                {
                    // check if there is anything other than whitespace between the end of the node and the trivia
                    let node_end = span.end.offset;
                    let trivia_start = t.span.start.offset;
                    let has_whitespace = self.source_text[node_end..trivia_start].chars().all(char::is_whitespace);
                    if !has_whitespace {
                        return None;
                    }

                    Some(t)
                } else {
                    None
                }
            })
            .any(|t| t.kind.is_single_line_comment())
    }

    #[must_use]
    pub(crate) fn has_trailing_comments(&self, span: Span) -> bool {
        let ending_line = self.source.line_number(span.end.offset);
        self.trivias
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if span.end.offset <= t.span.start.offset
                    && !self.used_trivia_indices.contains(&i)
                    && self.source.line_number(t.span.start.offset) == ending_line
                {
                    // check if there is anything other than whitespace between the end of the node and the trivia
                    let node_end = span.end.offset;
                    let trivia_start = t.span.start.offset;
                    let has_whitespace = self.source_text[node_end..trivia_start].chars().all(char::is_whitespace);
                    if !has_whitespace {
                        return None;
                    }

                    Some(t)
                } else {
                    None
                }
            })
            .any(|t| t.kind.is_comment())
    }

    #[must_use]
    pub(crate) fn print_trailing_comments(&mut self, span: Span, spaced: bool) -> Option<Document<'a>> {
        // collect all the trivias that start at the same line as the span ends.
        let ending_line = self.source.line_number(span.end.offset);
        let trivias: Vec<_> = self
            .trivias
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if span.end.offset <= t.span.start.offset
                    && !self.used_trivia_indices.contains(&i)
                    && self.source.line_number(t.span.start.offset) == ending_line
                {
                    // check if there is anything other than whitespace between the end of the node and the trivia
                    let node_end = span.end.offset;
                    let trivia_start = t.span.start.offset;
                    let has_whitespace = self.source_text[node_end..trivia_start].chars().all(char::is_whitespace);
                    if !has_whitespace {
                        return None;
                    }

                    self.used_trivia_indices.insert(i);

                    Some(t)
                } else {
                    None
                }
            })
            .copied()
            .collect();

        self.write_trivias(&trivias, spaced, true)
    }

    fn write_trivias(&mut self, trivias: &[Trivia], spaced: bool, trailing_new_line: bool) -> Option<Document<'a>> {
        let first_comment = trivias.iter().position(|t| t.kind.is_comment());
        // If no comments found, return None
        let Some(first) = first_comment else {
            return None;
        };

        let mut parts = if spaced { vec![space!()] } else { vec![] };
        let trivias = &trivias[first..];
        let len = trivias.len();
        let mut last_was_nl = false;
        for (i, t) in trivias.iter().enumerate() {
            let value = self.lookup(&t.value);
            if t.kind.is_comment() {
                // comments are always printed as is
                parts.push(self.format_comment(t.kind, value));
                last_was_nl = false;
            } else {
                if !trailing_new_line && i == (len - 1) {
                    break;
                }

                if value.contains('\n') || value.contains('\r') {
                    last_was_nl = true;
                    let newlines_to_add = Self::count_newlines_in_slice(value).min(2);
                    for _ in 0..newlines_to_add {
                        parts.push(Document::Line(Line::hardline()));
                    }
                } else if !last_was_nl {
                    parts.push(space!());
                    last_was_nl = false;
                }
            }
        }

        Some(array!(@parts))
    }

    fn format_comment(&mut self, trivia: TriviaKind, comment: &'a str) -> Document<'a> {
        // Determine the comment delimiters
        let (start_delim, end_delim) = match trivia {
            TriviaKind::MultiLineComment => ("/*", " */"),
            TriviaKind::DocBlockComment => ("/**", " */"),
            _ => {
                return static_str!(comment.trim_end());
            }
        };

        // Extract the content inside the delimiters
        let content = &comment[start_delim.len()..comment.len() - end_delim.len()];

        // Split into lines
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        if total_lines == 0 {
            // Empty comment
            let comment_str = format!("{}{}", start_delim, end_delim);
            static_str!(self.as_str(comment_str))
        } else if total_lines == 1 {
            // Single-line comment
            let line = lines[0].trim();
            if line.is_empty() {
                // Empty comment
                let comment_str = format!("{}{}", start_delim, end_delim);
                static_str!(self.as_str(comment_str))
            } else {
                let comment_str = format!("{} {}{}", start_delim, line, end_delim);
                static_str!(self.as_str(comment_str))
            }
        } else {
            // Multi-line comment
            let mut parts = vec![static_str!(start_delim)];

            // Handle the first line after '/*' or '/**'
            let first_line = lines[0].trim_end();
            if !first_line.is_empty() {
                parts.push(static_str!(self.as_str(first_line)));
            }

            parts.push(Document::Line(Line::hardline()));

            // Handle the rest of the lines
            for (idx, line) in lines.iter().enumerate().skip(1) {
                let is_last_line = idx == total_lines - 1;
                let trimmed_line = line.trim();

                // Remove leading '*' if present
                let content_line = if trimmed_line.starts_with('*') {
                    if let Some(" ") = trimmed_line.get(1..2) {
                        &trimmed_line[2..]
                    } else {
                        &trimmed_line[1..]
                    }
                } else {
                    trimmed_line
                };

                // Decide whether to add the line
                if !content_line.is_empty() || !is_last_line {
                    // Prepend '* ' if the line is not empty
                    let formatted_line =
                        if !content_line.is_empty() { format!(" * {}", content_line) } else { " *".to_string() };

                    parts.push(static_str!(self.as_str(formatted_line)));
                    parts.push(Document::Line(Line::hardline()));
                }
            }

            parts.push(static_str!(end_delim));

            array!(@parts)
        }
    }
}
