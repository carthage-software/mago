use std::borrow::Cow;

use fennec_span::Span;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::document::Separator;
use crate::Formatter;

use crate::comment::Comment;
use crate::comment::CommentFlags;

impl<'a> Formatter<'a> {
    #[must_use]
    pub(crate) fn print_comments(
        &mut self,
        before: Option<Document<'a>>,
        document: Document<'a>,
        after: Option<Document<'a>>,
    ) -> Document<'a> {
        match (before, after) {
            (Some(before), Some(after)) => Document::Array(vec![before, document, after]),
            (Some(before), None) => Document::Array(vec![before, document]),
            (None, Some(after)) => Document::Array(vec![document, after]),
            (None, None) => document,
        }
    }

    pub(crate) fn has_leading_own_line_comment(&self, range: Span) -> bool {
        self.has_comment_with_filter(range, CommentFlags::Leading, |comment| {
            self.has_newline(comment.end, /* backwards */ false)
        })
    }

    pub(crate) fn has_comment(&self, range: Span, flags: CommentFlags) -> bool {
        self.has_comment_with_filter(range, flags, |_| true)
    }

    pub(crate) fn has_comment_with_filter<F>(&self, range: Span, flags: CommentFlags, filter: F) -> bool
    where
        F: Fn(&Comment) -> bool,
    {
        let mut peekable_trivias = self.comments.clone();

        while let Some(comment) = peekable_trivias.peek() {
            let mut should_break = true;
            let comment = Comment::from_trivia(comment);

            if filter(&comment) {
                if comment.end <= range.start.offset {
                    if flags.contains(CommentFlags::Leading) && comment.matches_flags(flags) {
                        return true;
                    }

                    should_break = false;
                } else if range.end.offset < comment.start
                    && self.source_text[range.end.offset..comment.start]
                        .chars()
                        .all(|c| c == ' ' || c == ';' || c == ',')
                {
                    if flags.contains(CommentFlags::Trailing) && comment.matches_flags(flags) {
                        return true;
                    }

                    should_break = false;
                } else if comment.end <= range.end.offset {
                    if flags.contains(CommentFlags::Dangling) && comment.matches_flags(flags) {
                        return true;
                    }

                    should_break = false;
                }
            }

            if should_break {
                break;
            }

            peekable_trivias.next();
        }

        false
    }

    #[must_use]
    pub(crate) fn print_leading_comments(&mut self, range: Span) -> Option<Document<'a>> {
        let mut parts = vec![];
        while let Some(comment) = self.comments.peek() {
            let comment = Comment::from_trivia(comment);
            // Comment before the span
            if comment.end <= range.start.offset {
                self.comments.next();
                self.print_leading_comment(&mut parts, comment);
            } else {
                break;
            }
        }

        if parts.is_empty() {
            return None;
        }

        Some(Document::Array(parts))
    }

    fn print_leading_comment(&mut self, parts: &mut Vec<Document<'a>>, comment: Comment) {
        let printed = self.print_comment(comment);
        parts.push(printed);

        if comment.is_block {
            if self.has_newline(comment.end, /* backwards */ false) {
                if self.has_newline(comment.start, /* backwards */ true) {
                    parts.push(Document::BreakParent);
                    parts.push(Document::Line(Line::hardline()));
                } else {
                    parts.push(Document::Line(Line::default()));
                }
            } else {
                parts.push(Document::space());
            };
        } else {
            parts.push(Document::BreakParent);
            parts.push(Document::Line(Line::hardline()));
        }

        if self
            .skip_spaces(Some(comment.end), false)
            .and_then(|idx| self.skip_newline(Some(idx), false))
            .is_some_and(|i| self.has_newline(i, /* backwards */ false))
        {
            parts.push(Document::BreakParent);
            parts.push(Document::Line(Line::hardline()));
        }
    }

    #[must_use]
    pub(crate) fn print_trailing_comments(&mut self, range: Span) -> Option<Document<'a>> {
        let mut parts = vec![];
        let mut previous_comment: Option<Comment> = None;
        while let Some(comment) = self.comments.peek() {
            let comment = Comment::from_trivia(comment);
            // Trailing comment if there is nothing in between.
            if range.end.offset < comment.start
                && self.source_text[range.end.offset..comment.start].chars().all(|c| c == ' ' || c == ';' || c == ',')
            {
                self.comments.next();
                let previous = self.print_trailing_comment(&mut parts, comment, previous_comment);
                previous_comment = Some(previous);
            } else {
                break;
            }
        }

        if parts.is_empty() {
            return None;
        }

        Some(Document::Array(parts))
    }

    fn print_trailing_comment(
        &mut self,
        parts: &mut Vec<Document<'a>>,
        comment: Comment,
        previous: Option<Comment>,
    ) -> Comment {
        let printed = self.print_comment(comment);

        if previous.is_some_and(|c| c.has_line_suffix && !c.is_block)
            || self.has_newline(comment.start, /* backwards */ true)
        {
            parts.push(printed);
            let suffix = {
                let mut parts = vec![Document::BreakParent, Document::Line(Line::hardline())];

                if self.is_previous_line_empty(comment.start) {
                    parts.push(Document::Line(Line::hardline()));
                }

                parts
            };

            parts.push(Document::LineSuffix(suffix));

            return comment.with_line_suffix(true);
        }

        if !comment.is_block || previous.is_some_and(|c| c.has_line_suffix) {
            parts.push(Document::Array(vec![
                Document::LineSuffix(vec![Document::space(), printed]),
                Document::BreakParent,
            ]));

            return comment.with_line_suffix(true);
        }

        parts.push(Document::Array(vec![Document::space(), printed]));

        comment.with_line_suffix(false)
    }

    #[must_use]
    pub(crate) fn print_inner_comment(&mut self, range: Span) -> Vec<Document<'a>> {
        let mut parts = vec![];
        while let Some(comment) = self.comments.peek() {
            let comment = Comment::from_trivia(comment);
            // Comment within the span
            if comment.start >= range.start.offset && comment.end <= range.end.offset {
                self.comments.next();
                parts.push(self.print_comment(comment));
            } else {
                break;
            }
        }

        parts
    }

    #[must_use]
    pub(crate) fn print_dangling_comments(&mut self, range: Span, indented: bool) -> Option<Document<'a>> {
        let mut parts = vec![];
        while let Some(comment) = self.comments.peek() {
            let comment = Comment::from_trivia(comment);
            // Comment within the span
            if comment.end <= range.end.offset {
                parts.push(self.print_comment(comment));

                self.comments.next();
            } else {
                break;
            }
        }

        if parts.is_empty() {
            return None;
        }

        let document = Document::Array(Document::join(parts, Separator::Hardline));

        Some(if indented {
            Document::Array(vec![
                Document::Indent(vec![Document::BreakParent, Document::Line(Line::hardline()), document]),
                Document::Line(Line::hardline()),
            ])
        } else {
            Document::Array(vec![document, Document::Line(Line::hardline())])
        })
    }

    #[must_use]
    fn print_comment(&self, comment: Comment) -> Document<'a> {
        let content = &self.source_text[comment.start..comment.end];

        if !comment.is_block {
            return Document::String(content);
        }

        if !content.contains('\n') && !content.contains('\r') {
            return Document::String(content);
        }

        let lines = content.lines().collect::<Vec<_>>();
        let mut contents = vec![];

        // Process each line according to the specified rules
        let mut processed_lines = Vec::with_capacity(lines.len());
        for (i, line) in lines.iter().enumerate() {
            let processed_line = if i == 0 {
                // First line stays as is
                Cow::Borrowed(*line)
            } else if line.trim().is_empty() {
                // If the line is empty, format it as " *"
                Cow::Borrowed(" *")
            } else if line.trim_start().starts_with('*') {
                // Replace leading whitespace with a single space
                let rest = line.trim_start();
                Cow::Owned(format!(" {}", rest))
            } else {
                // Line does not have '*' after whitespaces, leave it as is
                Cow::Borrowed(*line)
            };

            processed_lines.push(processed_line);
        }

        // Assemble contents with hardlines between lines
        for (i, processed_line) in processed_lines.iter().enumerate() {
            contents.push(Document::String(self.as_str(processed_line)));
            if i < processed_lines.len() - 1 {
                contents.push(Document::Line(Line::hardline()));
            }
        }

        Document::Group(Group::new(contents))
    }
}
