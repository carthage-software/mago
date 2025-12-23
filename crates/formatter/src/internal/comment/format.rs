use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;
use bumpalo::vec;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Node;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::document::Separator;
use crate::document::Space;
use crate::internal::FormatterState;
use crate::internal::comment::Comment;
use crate::internal::comment::CommentFlags;

impl<'arena> FormatterState<'_, 'arena> {
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
        let Some(first_char_offset) = self.skip_spaces(Some(span.end_offset()), false) else {
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

    /// Checks if a node has a trailing line comment on the same line.
    ///
    /// This is different from `is_followed_by_comment_on_next_line` which checks
    /// for comments on the subsequent line. This method detects trailing comments
    /// that are on the same line as the node (e.g., `} // comment`).
    ///
    /// # Arguments
    ///
    /// * `span` - The span of the node to check for same-line trailing comments.
    ///
    /// # Returns
    ///
    /// `true` if there's a line comment on the same line after the node, `false` otherwise.
    pub(crate) fn has_same_line_trailing_comment(&self, span: Span) -> bool {
        let Some(first_char_offset) = self.skip_spaces(Some(span.end_offset()), false) else {
            return false;
        };

        // If there's a newline before the next content, the comment is on the next line, not same line
        if self.has_newline(first_char_offset, /* backwards */ true) {
            return false;
        }

        let remaining = &self.source_text[first_char_offset as usize..];
        remaining.starts_with("//") || (remaining.starts_with('#') && !remaining.starts_with("#["))
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

            if comment.end <= range.start_offset() {
                if flags.contains(CommentFlags::Leading) && comment.matches_flags(flags) {
                    return true;
                }
            } else if range.end_offset() < comment.start && self.is_insignificant(range.end_offset(), comment.start) {
                if flags.contains(CommentFlags::Trailing) && comment.matches_flags(flags) {
                    return true;
                }
            } else if comment.end <= range.end_offset() {
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
            if comment.start > range.end_offset() {
                break;
            }

            if comment.start >= range.start_offset() && comment.end <= range.end_offset() {
                return true;
            }
        }

        false
    }

    #[must_use]
    pub(crate) fn print_trailing_comments_for_node(&mut self, node: Node<'_, '_>) -> Option<Document<'arena>> {
        self.print_trailing_comments(node.span())
    }

    #[must_use]
    pub(crate) fn print_leading_comments(&mut self, range: Span) -> Option<Document<'arena>> {
        let mut parts = vec![in self.arena];

        while let Some(trivia) = self.all_comments.get(self.next_comment_index) {
            let comment = Comment::from_trivia(self.file, trivia);

            if comment.end <= range.start_offset() {
                // Check if comment is in an ignore region - if so, preserve as-is
                if self.get_ignore_region_for(comment.start).is_some() {
                    self.print_preserved_leading_comment(&mut parts, comment);
                } else {
                    self.print_leading_comment(&mut parts, comment);
                }
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

            if range.end_offset() < comment.start && self.is_insignificant(range.end_offset(), comment.start) {
                let gap = &self.source_text[(range.end_offset() as usize)..(comment.start as usize)];
                if comment.is_block && gap.contains(',') {
                    break;
                }

                // Check if comment is in an ignore region - if so, preserve as-is
                if self.get_ignore_region_for(comment.start).is_some() {
                    self.print_preserved_trailing_comment(&mut parts, comment);
                    previous_comment = Some(comment);
                } else {
                    previous_comment =
                        Some(self.print_trailing_comment(&mut parts, comment, previous_comment, range.end_offset()));
                }
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
            // For inline comments (// or #), check if there's a newline after the comment.
            // If there is, add a hard line break; if not, add a soft space.
            // This handles cases like `<?//?><?` where the comment should stay on the same line.
            if self.has_newline(comment.end, /* backwards */ false) {
                Document::Array(
                    vec![in self.arena; self.print_comment(comment), Document::BreakParent, Document::Line(Line::hard())],
                )
            } else {
                Document::Array(vec![in self.arena; self.print_comment(comment), Document::Space(Space::soft())])
            }
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
        token_end_offset: u32,
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

        if !comment.is_block || previous.is_some_and(|c| c.has_line_suffix) {
            parts.push(Document::LineSuffix(vec![in self.arena; Document::Space(Space::soft()), printed]));

            return comment.with_line_suffix(true);
        }

        let followed_by_semicolon = self
            .skip_spaces(Some(comment.end), false)
            .map(|idx| self.source_text.as_bytes().get(idx as usize))
            .is_some_and(|c| c == Some(&b';'));

        let has_semicolon_in_gap =
            self.source_text[(token_end_offset as usize)..(comment.start as usize)].bytes().any(|c| c == b';');

        if followed_by_semicolon || has_semicolon_in_gap {
            parts.push(Document::LineSuffix(vec![in self.arena; Document::Space(Space::soft()), printed]));

            return comment.with_line_suffix(true);
        }

        parts.push(Document::Array(vec![in self.arena; Document::Space(Space::soft()), printed]));

        comment.with_line_suffix(false)
    }

    /// Prints a leading comment that is within an ignore region, preserving its original formatting.
    fn print_preserved_leading_comment(&mut self, parts: &mut Vec<'arena, Document<'arena>>, comment: Comment) {
        // Preserve the comment exactly as-is from source
        let preserved = self.get_source_slice(comment.start, comment.end);
        let comment_document = if comment.is_block {
            if self.has_newline(comment.end, /* backwards */ false) {
                if self.has_newline(comment.start, /* backwards */ true) {
                    Document::Array(vec![
                        in self.arena;
                        Document::String(preserved),
                        Document::BreakParent,
                        Document::Line(Line::hard()),
                    ])
                } else {
                    Document::Array(vec![in self.arena; Document::String(preserved), Document::Line(Line::default())])
                }
            } else {
                Document::Array(vec![in self.arena; Document::String(preserved), Document::Space(Space::soft())])
            }
        } else {
            Document::Array(
                vec![in self.arena; Document::String(preserved), Document::BreakParent, Document::Line(Line::hard())],
            )
        };

        parts.push(comment_document);

        // Check for blank lines after comment
        if self
            .skip_spaces(Some(comment.end), false)
            .and_then(|idx| self.skip_newline(Some(idx), false))
            .is_some_and(|i| self.has_newline(i, /* backwards */ false))
        {
            parts.push(Document::BreakParent);
            parts.push(Document::Line(Line::hard()));
        }
    }

    /// Prints a trailing comment that is within an ignore region, preserving its original formatting.
    fn print_preserved_trailing_comment(&mut self, parts: &mut Vec<'arena, Document<'arena>>, comment: Comment) {
        // Preserve the comment exactly as-is from source
        let preserved = self.get_source_slice(comment.start, comment.end);

        if self.has_newline(comment.start, /* backwards */ true) {
            parts.push(Document::String(preserved));
            let suffix = {
                let mut parts = vec![in self.arena; Document::BreakParent, Document::Line(Line::hard())];

                if self.is_previous_line_empty(comment.start) {
                    parts.push(Document::Line(Line::hard()));
                }

                parts
            };

            parts.push(Document::LineSuffix(suffix));
        } else if comment.is_inline_comment() {
            parts.push(Document::LineSuffix(
                vec![in self.arena; Document::Space(Space::soft()), Document::String(preserved)],
            ));
        } else {
            parts.push(Document::Array(
                vec![in self.arena; Document::Space(Space::soft()), Document::String(preserved)],
            ));
        }
    }

    #[must_use]
    pub(crate) fn print_inner_comment(&mut self, range: Span, should_indent: bool) -> Option<Document<'arena>> {
        let mut parts = vec![in self.arena];
        let mut must_break = false;
        let mut consumed_count = 0;

        for trivia in &self.all_comments[self.next_comment_index..] {
            let comment = Comment::from_trivia(self.file, trivia);

            if comment.start >= range.start_offset() && comment.end <= range.end_offset() {
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

            if comment.end <= range.end_offset() {
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

            if comment.start >= after.end_offset()
                && comment.end <= before.start_offset()
                && self.is_insignificant(after.end_offset(), comment.start)
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

    /// Prints trailing comments that appear between two nodes, suitable for use after `{`.
    /// Unlike `print_dangling_comments_between_nodes`, this uses LineSuffix for inline comments
    /// to keep them on the same line as the preceding content.
    #[must_use]
    pub(crate) fn print_trailing_comments_between_nodes(
        &mut self,
        after: Span,
        before: Span,
    ) -> Option<Document<'arena>> {
        let mut parts = vec![in self.arena];
        let mut previous_comment: Option<Comment> = None;

        while let Some(trivia) = self.all_comments.get(self.next_comment_index) {
            let comment = Comment::from_trivia(self.file, trivia);

            let is_between = comment.start >= after.end_offset() && comment.end <= before.start_offset();
            let gap_is_ok =
                after.end_offset() == comment.start || self.is_insignificant(after.end_offset(), comment.start);

            if is_between && gap_is_ok {
                previous_comment =
                    Some(self.print_trailing_comment(&mut parts, comment, previous_comment, after.end_offset()));
                self.next_comment_index += 1;
            } else {
                break;
            }
        }

        if parts.is_empty() { None } else { Some(Document::Array(parts)) }
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
