use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;
use bumpalo::vec;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::document::Separator;
use crate::document::Space;
use crate::internal::FormatterState;
use crate::internal::comment::Comment;
use crate::internal::comment::CommentFlags;
use crate::internal::utils::unwrap_parenthesized;

impl<'ctx, 'arena> FormatterState<'ctx, 'arena> {
    #[must_use]
    pub(crate) fn print_comments(
        &mut self,
        before: Option<Document<'arena>>,
        document: Document<'arena>,
        after: Option<Document<'arena>>,
    ) -> Document<'arena> {
        match (before, after) {
            (Some(before), Some(after)) => Document::Array(vec![in self.arena; before, document, after]),
            (Some(before), None) => Document::Array(vec![in self.arena; before, document]),
            (None, Some(after)) => Document::Array(vec![in self.arena; document, after]),
            (None, None) => document,
        }
    }

    /// Returns an iterator over the remaining, unconsumed comments.
    fn remaining_comments(&self) -> impl Iterator<Item = Comment> {
        self.all_comments[self.next_comment_index..].iter().map(|trivia| Comment::from_trivia(self.file, trivia))
    }

    /// Checks if a node is followed by a comment on its own line.
    ///
    /// # Arguments
    ///
    /// * `span` - The span of the node after which to check for a comment.
    ///
    /// # Returns
    ///
    /// `true` if the next line is a comment line, `false` otherwise.
    /// Checks if a node is followed by a comment on its own line.
    ///
    /// # Arguments
    ///
    /// * `span` - The span of the node after which to check for a comment.
    ///
    /// # Returns
    ///
    /// `true` if the next substantive line is a comment line, `false` otherwise.
    pub(crate) fn is_followed_by_comment_on_next_line(&self, span: Span) -> bool {
        let Some(first_char_offset) = self.skip_spaces(Some(span.end.offset), false) else {
            return false;
        };

        if !self.has_newline(first_char_offset, /* backwards */ false) {
            return false;
        }

        let Some(next_content_offset) = self.skip_spaces_and_new_lines(Some(first_char_offset), false) else {
            return false;
        };

        let remaining_content = &self.source_text[next_content_offset as usize..];

        remaining_content.starts_with("//")
            || remaining_content.starts_with("/*")
            || (remaining_content.starts_with('#') && !remaining_content.starts_with("#["))
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
        for comment in self.remaining_comments() {
            if !filter(&comment) {
                break;
            }

            if comment.end <= range.start.offset {
                if flags.contains(CommentFlags::Leading) && comment.matches_flags(flags) {
                    return true;
                }
            } else if range.end.offset < comment.start && self.is_insignificant(range.end.offset, comment.start) {
                if flags.contains(CommentFlags::Trailing) && comment.matches_flags(flags) {
                    return true;
                }
            } else if comment.end <= range.end.offset {
                if flags.contains(CommentFlags::Dangling) && comment.matches_flags(flags) {
                    return true;
                }
            } else {
                break;
            }
        }

        false
    }

    #[must_use]
    #[inline]
    pub fn has_inner_comment(&self, range: Span) -> bool {
        for comment in self.remaining_comments() {
            if comment.start > range.end.offset {
                break;
            }

            if comment.start >= range.start.offset && comment.end <= range.end.offset {
                return true;
            }
        }

        false
    }

    #[must_use]
    pub(crate) fn print_trailing_comments_for_node(&mut self, node: Node<'_, '_>) -> Option<Document<'arena>> {
        let range = match node {
            Node::ArrowFunction(arrow_function) if self.in_pipe_chain_arrow_segment => {
                let mut value = unwrap_parenthesized(arrow_function.expression);
                while let Expression::Pipe(pipe) = value {
                    value = unwrap_parenthesized(pipe.input);
                }

                value.span()
            }
            _ => node.span(),
        };

        self.print_trailing_comments(range)
    }

    #[must_use]
    pub(crate) fn print_leading_comments(&mut self, range: Span) -> Option<Document<'arena>> {
        let mut parts = vec![in self.arena];

        while let Some(trivia) = self.all_comments.get(self.next_comment_index) {
            let comment = Comment::from_trivia(self.file, trivia);

            if comment.end <= range.start.offset {
                self.print_leading_comment(&mut parts, comment);
                self.next_comment_index += 1;
            } else {
                break;
            }
        }

        if parts.is_empty() { None } else { Some(Document::Array(parts)) }
    }

    #[must_use]
    pub(crate) fn print_trailing_comments(&mut self, range: Span) -> Option<Document<'arena>> {
        let mut parts = vec![in self.arena];
        let mut previous_comment: Option<Comment> = None;

        while let Some(trivia) = self.all_comments.get(self.next_comment_index) {
            let comment = Comment::from_trivia(self.file, trivia);

            if range.end.offset < comment.start && self.is_insignificant(range.end.offset, comment.start) {
                let previous = self.print_trailing_comment(&mut parts, comment, previous_comment);
                previous_comment = Some(previous);
                self.next_comment_index += 1;
            } else {
                break;
            }
        }

        if parts.is_empty() { None } else { Some(Document::Array(parts)) }
    }

    fn print_leading_comment(&mut self, parts: &mut Vec<'arena, Document<'arena>>, comment: Comment) {
        let comment_document = if comment.is_block {
            if self.has_newline(comment.end, /* backwards */ false) {
                if self.has_newline(comment.start, /* backwards */ true) {
                    Document::Array(vec![
                        in self.arena;
                        self.print_comment(comment),
                        Document::BreakParent,
                        Document::Line(Line::hard()),
                    ])
                } else {
                    Document::Array(vec![in self.arena; self.print_comment(comment), Document::Line(Line::default())])
                }
            } else {
                Document::Array(vec![in self.arena; self.print_comment(comment), Document::Space(Space::soft())])
            }
        } else {
            Document::Array(
                vec![in self.arena; self.print_comment(comment), Document::BreakParent, Document::Line(Line::hard())],
            )
        };

        parts.push(comment_document);

        if self
            .skip_spaces(Some(comment.end), false)
            .and_then(|idx| self.skip_newline(Some(idx), false))
            .is_some_and(|i| self.has_newline(i, /* backwards */ false))
        {
            parts.push(Document::BreakParent);
            parts.push(Document::Line(Line::hard()));
        }
    }

    fn print_trailing_comment(
        &mut self,
        parts: &mut Vec<'arena, Document<'arena>>,
        comment: Comment,
        previous: Option<Comment>,
    ) -> Comment {
        let printed = self.print_comment(comment);

        if previous.is_some_and(|c| c.has_line_suffix && !c.is_inline_comment())
            || self.has_newline(comment.start, /* backwards */ true)
        {
            parts.push(printed);
            let suffix = {
                let mut parts = vec![in self.arena; Document::BreakParent, Document::Line(Line::hard())];

                if self.is_previous_line_empty(comment.start) {
                    parts.push(Document::Line(Line::hard()));
                }

                parts
            };

            parts.push(Document::LineSuffix(suffix));

            return comment.with_line_suffix(true);
        }

        if comment.is_inline_comment() || previous.is_some_and(|c| c.has_line_suffix) {
            parts.push(Document::LineSuffix(vec![in self.arena; Document::Space(Space::soft()), printed]));

            return comment.with_line_suffix(true);
        }

        parts.push(Document::Array(vec![in self.arena; Document::Space(Space::soft()), printed]));

        comment.with_line_suffix(false)
    }

    #[must_use]
    pub(crate) fn print_inner_comment(&mut self, range: Span, should_indent: bool) -> Option<Document<'arena>> {
        let mut parts = vec![in self.arena];
        let mut must_break = false;
        let mut consumed_count = 0;

        for trivia in &self.all_comments[self.next_comment_index..] {
            let comment = Comment::from_trivia(self.file, trivia);

            if comment.start >= range.start.offset && comment.end <= range.end.offset {
                must_break = must_break || !comment.is_block;
                if !should_indent && self.is_next_line_empty(trivia.span) {
                    parts.push(Document::Array(
                        vec![in self.arena; self.print_comment(comment), Document::Line(Line::hard())],
                    ));
                    must_break = true;
                } else {
                    parts.push(self.print_comment(comment));
                }
                consumed_count += 1;
            } else {
                break;
            }
        }

        if consumed_count > 0 {
            self.next_comment_index += consumed_count;
        }

        if parts.is_empty() {
            return None;
        }

        let document = Document::Array(Document::join(self.arena, parts, Separator::HardLine));

        Some(if should_indent {
            Document::Group(
                Group::new(vec![
                    in self.arena;
                    Document::Indent(vec![in self.arena; Document::Line(Line::default()), document]),
                    Document::Line(Line::default()),
                ])
                .with_break(must_break),
            )
        } else {
            Document::Group(
                Group::new(vec![
                    in self.arena;
                    Document::Array(vec![in self.arena; Document::Line(Line::default()), document]),
                    Document::Line(Line::default()),
                ])
                .with_break(must_break),
            )
        })
    }

    #[must_use]
    pub(crate) fn print_dangling_comments(&mut self, range: Span, indented: bool) -> Option<Document<'arena>> {
        let mut parts = vec![in self.arena];
        let mut consumed_count = 0;

        // Iterate over the remaining comment slice.
        for trivia in &self.all_comments[self.next_comment_index..] {
            let comment = Comment::from_trivia(self.file, trivia);

            if comment.end <= range.end.offset {
                if !indented && self.is_next_line_empty(trivia.span) {
                    parts.push(Document::Array(
                        vec![in self.arena; self.print_comment(comment), Document::Line(Line::hard())],
                    ));
                } else {
                    parts.push(self.print_comment(comment));
                }
                consumed_count += 1;
            } else {
                break;
            }
        }

        if consumed_count > 0 {
            self.next_comment_index += consumed_count;
        }

        if parts.is_empty() {
            return None;
        }

        let document = Document::Array(Document::join(self.arena, parts, Separator::HardLine));

        Some(if indented {
            Document::Array(vec![
                in self.arena;
                Document::Indent(vec![in self.arena; Document::BreakParent, Document::Line(Line::hard()), document]),
                Document::Line(Line::hard()),
            ])
        } else {
            Document::Array(vec![in self.arena; document, Document::Line(Line::hard())])
        })
    }

    #[must_use]
    pub(crate) fn print_dangling_comments_between_nodes(
        &mut self,
        after: Span,
        before: Span,
    ) -> Option<Document<'arena>> {
        let mut parts = vec![in self.arena];
        let mut consumed_count = 0;

        // Iterate over the remaining comment slice.
        for trivia in &self.all_comments[self.next_comment_index..] {
            let comment = Comment::from_trivia(self.file, trivia);

            if comment.start >= after.end.offset
                && comment.end <= before.start.offset
                && self.is_insignificant(after.end.offset, comment.start)
            {
                parts.push(self.print_comment(comment));
                consumed_count += 1;
            } else {
                break;
            }
        }

        if consumed_count > 0 {
            self.next_comment_index += consumed_count;
        }

        if parts.is_empty() {
            return None;
        }

        Some(Document::Indent(vec![
            in self.arena;
            Document::BreakParent,
            Document::Line(Line::hard()),
            Document::Array(Document::join(self.arena, parts, Separator::HardLine)),
        ]))
    }

    #[must_use]
    fn print_comment(&self, comment: Comment) -> Document<'arena> {
        let content = &self.source_text[comment.start as usize..comment.end as usize];

        if comment.is_inline_comment() {
            if !comment.is_single_line {
                return Document::String(content);
            }

            let new_content = if comment.is_shell_comment {
                let mut buf = Vec::with_capacity_in(content.len() + 2, self.arena);
                buf.extend_from_slice(b"// ");
                buf.extend_from_slice(content[1..].trim().as_bytes());

                // SAFETY: We are constructing the string from valid UTF-8 parts.
                unsafe { std::str::from_utf8_unchecked(buf.into_bump_slice()) }
            } else {
                content
            };

            return Document::String(new_content);
        }

        if !content.contains('\n') && !content.contains('\r') {
            return Document::String(content);
        }

        let lines = content.lines().collect_in::<Vec<_>>(self.arena);
        let mut contents = Vec::with_capacity_in(lines.len() * 2, self.arena);

        let should_add_asterisks = if content.starts_with("/**") {
            true
        } else {
            let content_lines = &lines[1..lines.len() - 1];

            let potential_prefix = content_lines
                .iter()
                .map(|line| line.trim_start())
                .find(|trimmed| !trimmed.is_empty())
                .and_then(|first_line| first_line.chars().next());

            if let Some(prefix_char) = potential_prefix {
                if !prefix_char.is_alphanumeric() && prefix_char != '*' {
                    let all_lines_match = content_lines.iter().all(|line| line.trim_start().starts_with(prefix_char));

                    !all_lines_match
                } else {
                    true
                }
            } else {
                true
            }
        };

        for (i, line) in lines.iter().enumerate() {
            let trimmed_line = line.trim_start();

            let processed_line = if i == 0 {
                *line
            } else if !should_add_asterisks {
                let mut buf = Vec::with_capacity_in(trimmed_line.len() + 1, self.arena);
                buf.push(b' ');
                buf.extend_from_slice(trimmed_line.trim_end().as_bytes());
                unsafe { std::str::from_utf8_unchecked(buf.into_bump_slice()) }
            } else if trimmed_line.is_empty() {
                " *"
            } else if trimmed_line.starts_with('*') {
                let mut buf = Vec::with_capacity_in(trimmed_line.len() + 1, self.arena);
                buf.push(b' ');
                buf.extend_from_slice(trimmed_line.trim_end().as_bytes());
                unsafe { std::str::from_utf8_unchecked(buf.into_bump_slice()) }
            } else {
                let mut buf = Vec::with_capacity_in(trimmed_line.len() + 3, self.arena);
                buf.extend_from_slice(b" * ");
                buf.extend_from_slice(trimmed_line.trim_end().as_bytes());
                unsafe { std::str::from_utf8_unchecked(buf.into_bump_slice()) }
            };

            contents.push(Document::String(processed_line));
            if i < lines.len() - 1 {
                contents.push(Document::Line(Line::hard()));
            }
        }

        Document::Group(Group::new(contents))
    }
}
