use bumpalo::Bump;
use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec as BumpVec;

use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_php_version::PHPVersion;
use mago_span::Span;
use mago_syntax::ast::Node;
use mago_syntax::ast::Program;
use mago_syntax::ast::Trivia;

use crate::document::group::GroupIdentifier;
use crate::document::group::GroupIdentifierBuilder;
use crate::internal::format::assignment::AssignmentAlignment;
use crate::settings::FormatSettings;

pub mod comment;
pub mod format;
pub mod macros;
pub mod parens;
pub mod printer;
pub mod utils;

#[derive(Debug, Clone, Copy, Default)]
pub struct ArgumentState {
    expand_first_argument: bool,
    expand_last_argument: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ParameterState {
    force_break: bool,
}

/// A region of source code that should not be formatted.
#[derive(Debug, Clone, Copy)]
pub struct IgnoreRegion {
    /// Start offset (beginning of the @mago-format-ignore-start comment)
    pub start: u32,
    /// End offset (end of the @mago-format-ignore-end comment, or EOF)
    pub end: u32,
}

/// Markers that indicate the start of an unformatted region.
const IGNORE_START_MARKERS: [&str; 2] = ["@mago-format-ignore-start", "@mago-formatter-ignore-start"];

/// Markers that indicate the end of an unformatted region.
const IGNORE_END_MARKERS: [&str; 2] = ["@mago-format-ignore-end", "@mago-formatter-ignore-end"];

/// Markers that indicate the next statement should be ignored.
const IGNORE_NEXT_MARKERS: [&str; 2] = ["@mago-format-ignore-next", "@mago-formatter-ignore-next"];

/// A marker indicating the next statement should not be formatted.
#[derive(Debug, Clone, Copy)]
pub struct IgnoreNextMarker {
    /// Start offset of the comment containing the marker.
    pub comment_start: u32,
    /// End offset of the comment containing the marker.
    pub comment_end: u32,
}

/// Scans comments to build a list of ignore regions and ignore-next markers.
fn build_ignore_markers<'arena>(
    arena: &'arena Bump,
    comments: &[Trivia<'arena>],
    source_len: u32,
) -> (&'arena [IgnoreRegion], &'arena [IgnoreNextMarker]) {
    let mut regions = BumpVec::new_in(arena);
    let mut next_markers = BumpVec::new_in(arena);
    let mut current_start: Option<u32> = None;

    for comment in comments {
        let has_start = IGNORE_START_MARKERS.iter().any(|m| comment.value.contains(m));
        let has_end = IGNORE_END_MARKERS.iter().any(|m| comment.value.contains(m));
        let has_next = IGNORE_NEXT_MARKERS.iter().any(|m| comment.value.contains(m));

        if has_start && current_start.is_none() {
            current_start = Some(comment.span.start.offset);
        } else if has_end && current_start.is_some() {
            regions.push(IgnoreRegion { start: current_start.unwrap(), end: comment.span.end.offset });
            current_start = None;
        }

        if has_next {
            next_markers.push(IgnoreNextMarker {
                comment_start: comment.span.start.offset,
                comment_end: comment.span.end.offset,
            });
        }
    }

    // If start without end, extend to EOF
    if let Some(start) = current_start {
        regions.push(IgnoreRegion { start, end: source_len });
    }

    (regions.into_bump_slice(), next_markers.into_bump_slice())
}

#[derive(Debug)]
pub struct FormatterState<'ctx, 'arena> {
    arena: &'arena Bump,
    source_text: &'arena str,
    file: &'ctx File,
    php_version: PHPVersion,
    settings: FormatSettings,
    stack: BumpVec<'arena, Node<'arena, 'arena>>,
    all_comments: &'arena [Trivia<'arena>],
    next_comment_index: usize,
    ignore_regions: &'arena [IgnoreRegion],
    ignore_next_markers: &'arena [IgnoreNextMarker],
    next_ignore_next_index: usize,
    scripting_mode: bool,
    id_builder: GroupIdentifierBuilder,
    argument_state: ArgumentState,
    parameter_state: ParameterState,
    in_script_terminating_statement: bool,
    in_condition: bool,
    is_wrapped_in_parens: bool,
    is_in_inlined_binary_chain: bool,
    halted_compilation: bool,
    alignment_context: Option<AssignmentAlignment>,
}

impl<'ctx, 'arena> FormatterState<'ctx, 'arena> {
    pub fn new(
        arena: &'arena Bump,
        program: &'arena Program<'arena>,
        file: &'ctx File,
        php_version: PHPVersion,
        settings: FormatSettings,
    ) -> Self {
        let all_comments = program
            .trivia
            .iter()
            .filter(|t| t.kind.is_comment())
            .copied()
            .collect_in::<BumpVec<_>>(arena)
            .into_bump_slice();

        let (ignore_regions, ignore_next_markers) =
            build_ignore_markers(arena, all_comments, program.source_text.len() as u32);

        Self {
            arena,
            file,
            source_text: program.source_text,
            php_version,
            settings,
            stack: BumpVec::new_in(arena),
            all_comments,
            next_comment_index: 0,
            ignore_regions,
            ignore_next_markers,
            next_ignore_next_index: 0,
            scripting_mode: false,
            id_builder: GroupIdentifierBuilder::new(),
            argument_state: ArgumentState::default(),
            parameter_state: ParameterState::default(),
            in_condition: false,
            is_wrapped_in_parens: false,
            is_in_inlined_binary_chain: false,
            halted_compilation: false,
            in_script_terminating_statement: false,
            alignment_context: None,
        }
    }

    /// Get the current alignment context.
    #[inline]
    pub fn alignment_context(&self) -> Option<AssignmentAlignment> {
        self.alignment_context
    }

    /// Set the alignment context for the current formatting operation.
    #[inline]
    pub fn set_alignment_context(&mut self, context: Option<AssignmentAlignment>) {
        self.alignment_context = context;
    }

    /// Get the current next comment index.
    #[inline]
    pub(crate) fn get_next_comment_index(&self) -> usize {
        self.next_comment_index
    }

    /// Set the next comment index (used when formatting sorted members).
    #[inline]
    pub(crate) fn set_next_comment_index(&mut self, index: usize) {
        self.next_comment_index = index;
    }

    /// Get the all_comments slice.
    #[inline]
    pub(crate) fn all_comments(&self) -> &'arena [Trivia<'arena>] {
        self.all_comments
    }


    fn next_id(&mut self) -> GroupIdentifier {
        self.id_builder.next_id()
    }

    #[inline]
    fn as_str(&self, string: impl AsRef<str>) -> &'arena str {
        self.arena.alloc_str(string.as_ref())
    }

    #[inline]
    fn enter_node(&mut self, node: Node<'arena, 'arena>) {
        self.stack.push(node);
    }

    #[inline]
    fn leave_node(&mut self) {
        self.stack.pop();
    }

    #[inline]
    fn current_node(&self) -> Node<'arena, 'arena> {
        self.stack[self.stack.len() - 1]
    }

    #[inline]
    fn parent_node(&self) -> Node<'arena, 'arena> {
        self.stack[self.stack.len() - 2]
    }

    #[inline]
    fn grandparent_node(&self) -> Option<Node<'arena, 'arena>> {
        let len = self.stack.len();

        (len > 2).then(|| self.stack[len - 2 - 1])
    }

    #[inline]
    fn great_grandparent_node(&self) -> Option<Node<'arena, 'arena>> {
        let len = self.stack.len();
        (len > 3).then(|| self.stack[len - 3 - 1])
    }

    #[inline]
    fn nth_parent_kind(&self, n: u32) -> Option<Node<'arena, 'arena>> {
        let n = n as usize;
        let len = self.stack.len();

        (len > n).then(|| self.stack[len - n - 1])
    }

    #[inline]
    fn is_previous_line_empty(&self, start_index: u32) -> bool {
        let idx = start_index - 1;
        let idx = self.skip_spaces(Some(idx), true);
        let idx = self.skip_newline(idx, true);
        let idx = self.skip_spaces(idx, true);
        let idx2 = self.skip_newline(idx, true);
        idx != idx2
    }

    /// Checks if a span is at the start of its line, ignoring leading whitespace.
    ///
    /// For example, given the code `  $foo = 1;`, a span for `$foo` would
    /// return `true`, but a span for `= 1` would return `false`.
    ///
    /// # Returns
    ///
    /// `true` if there are no non-whitespace characters on the line before the span's start.
    pub(crate) fn is_at_start_of_line(&self, span: Span) -> bool {
        let line_index = self.file.line_number(span.start.offset);
        let line_start_offset = self.file.lines[line_index as usize] as usize;
        let span_start_offset = span.start.offset as usize;
        let prefix = &self.source_text[line_start_offset..span_start_offset];

        prefix.trim().is_empty()
    }

    #[inline]
    fn is_next_line_empty(&self, span: Span) -> bool {
        self.is_next_line_empty_after_index(span.end.offset)
    }

    #[inline]
    pub(crate) fn is_next_line_empty_after_index(&self, start_index: u32) -> bool {
        let mut old_idx = None;
        let mut idx = Some(start_index);
        while idx != old_idx {
            old_idx = idx;
            idx = self.skip_to_line_end(idx);
            idx = self.skip_spaces(idx, /* backwards */ false);
        }

        idx = self.skip_inline_comments(idx);
        idx = self.skip_newline(idx, /* backwards */ false);
        idx.is_some_and(|idx| self.has_newline(idx, /* backwards */ false))
    }

    #[inline]
    fn skip_inline_comments(&self, start_index: Option<u32>) -> Option<u32> {
        let start_index = start_index?;
        let start_index_usize = start_index as usize;
        if start_index_usize + 1 >= self.source_text.len() {
            return Some(start_index); // Not enough characters to check for comment
        }

        if self.source_text[start_index_usize..].starts_with("//")
            || (self.source_text[start_index_usize..].starts_with('#')
                && !self.source_text[start_index_usize + 1..].starts_with('['))
        {
            return self.skip_to_line_end_or_closing_tag(Some(start_index));
        }

        if self.source_text[start_index_usize..].starts_with("/*") {
            // Find the closing */
            if let Some(end_pos) = self.source_text[start_index_usize + 2..].find("*/") {
                let end_index = start_index_usize + 2 + end_pos + 2; // +2 for the "*/" itself

                // Check if there's a newline between /* and */
                let comment_text = &self.source_text[start_index_usize..end_index];
                if !comment_text.contains('\n') && !comment_text.contains('\r') {
                    return Some(end_index as u32);
                }

                // If there's a newline, we don't consider it an inline comment
                // so we don't skip it
            }
        }

        Some(start_index)
    }

    #[inline]
    fn is_insignificant(&self, start_index: u32, end_index: u32) -> bool {
        let start_index = start_index as usize;
        let end_index = end_index as usize;

        if start_index >= end_index || end_index > self.source_text.len() {
            return false;
        }

        self.source_text[start_index..end_index].bytes().all(is_insignificant)
    }

    #[inline]
    fn skip_to_line_end(&self, start_index: Option<u32>) -> Option<u32> {
        let mut index = self.skip(start_index, false, is_insignificant);
        index = self.skip_inline_comments(index);
        index
    }

    #[inline]
    fn skip_spaces(&self, start_index: Option<u32>, backwards: bool) -> Option<u32> {
        self.skip(start_index, backwards, is_space)
    }

    #[inline]
    fn skip_spaces_and_new_lines(&self, start_index: Option<u32>, backwards: bool) -> Option<u32> {
        self.skip(start_index, backwards, is_line_terminator_or_space)
    }

    #[inline]
    fn skip<F>(&self, start_index: Option<u32>, backwards: bool, f: F) -> Option<u32>
    where
        F: Fn(u8) -> bool,
    {
        let start_index = start_index? as usize;
        let mut index = start_index;
        if backwards {
            for c in self.source_text[..=start_index].bytes().rev() {
                if !f(c) {
                    return Some(index as u32);
                }
                index -= 1;
            }
        } else {
            let source_bytes = self.source_text.as_bytes();
            let text_len = source_bytes.len();
            while index < text_len {
                if !f(source_bytes[index]) {
                    return Some(index as u32);
                }

                index += 1;
            }
        }

        None
    }

    /// Skips forward from a starting index until a newline or a `?>` tag is found.
    #[inline]
    fn skip_to_line_end_or_closing_tag(&self, start_index: Option<u32>) -> Option<u32> {
        let mut index = start_index? as usize;
        let source_bytes = self.source_text.as_bytes();
        let text_len = source_bytes.len();

        while index < text_len {
            let byte = source_bytes[index];

            // Stop if we find a newline character.
            if is_line_terminator(byte) {
                return Some(index as u32);
            }

            // Stop if we find a `?>` sequence.
            if byte == b'?' && index + 1 < text_len && source_bytes[index + 1] == b'>' {
                return Some(index as u32);
            }

            index += 1;
        }

        None
    }

    #[inline]
    fn skip_newline(&self, start_index: Option<u32>, backwards: bool) -> Option<u32> {
        let start_index = start_index?;
        let start_index_usize = start_index as usize;
        let c = if backwards {
            self.source_text[..=start_index_usize].bytes().next_back()
        } else {
            self.source_text[start_index_usize..].bytes().next()
        }?;

        if matches!(c, b'\n') {
            return Some(if backwards { start_index - 1 } else { start_index + 1 });
        }

        if matches!(c, b'\r') {
            let next_index = if backwards { start_index_usize - 1 } else { start_index_usize + 1 };
            let next_c = if backwards {
                self.source_text[..=next_index].bytes().next_back()
            } else {
                self.source_text[next_index..].bytes().next()
            }?;

            if matches!(next_c, b'\n') {
                return Some(if backwards { start_index - 2 } else { start_index + 2 });
            }
        }

        Some(start_index)
    }

    #[inline]
    fn has_newline(&self, start_index: u32, backwards: bool) -> bool {
        if (backwards && start_index == 0) || (!backwards && (start_index as usize) == self.source_text.len()) {
            return false;
        }
        let start_index = if backwards { start_index - 1 } else { start_index };
        let idx = self.skip_spaces(Some(start_index), backwards);
        let idx2 = self.skip_newline(idx, backwards);
        idx != idx2
    }

    #[inline]
    fn split_lines(&self, slice: &'arena str) -> BumpVec<'arena, &'arena str> {
        let mut lines = BumpVec::new_in(self.arena);
        let mut remaining = slice;

        while !remaining.is_empty() {
            if let Some(pos) = remaining.find("\r\n") {
                lines.push(&remaining[..pos]);
                remaining = &remaining[pos + 2..];
            } else if let Some(pos) = remaining.find('\n') {
                lines.push(&remaining[..pos]);
                remaining = &remaining[pos + 1..];
            } else {
                // No more newlines
                if !remaining.is_empty() {
                    lines.push(remaining);
                }
                break;
            }
        }

        lines
    }

    #[inline]
    fn skip_leading_whitespace_up_to(s: &'arena str, indent: usize) -> &'arena str {
        let mut position = 0;
        for (count, (i, b)) in s.bytes().enumerate().enumerate() {
            // Check if the current byte represents whitespace
            if !b.is_ascii_whitespace() || count >= indent {
                break;
            }

            position = i + 1;
        }

        &s[position..]
    }

    /// Returns the ignore region that contains this offset, if any.
    pub(crate) fn get_ignore_region_for(&self, offset: u32) -> Option<&IgnoreRegion> {
        self.ignore_regions.iter().find(|r| offset >= r.start && offset < r.end)
    }

    /// Get source text slice for a given range.
    pub(crate) fn get_source_slice(&self, start: u32, end: u32) -> &'arena str {
        &self.source_text[start as usize..end as usize]
    }

    /// Advance the comment index past a given offset.
    pub(crate) fn skip_comments_until(&mut self, end_offset: u32) {
        while let Some(comment) = self.all_comments.get(self.next_comment_index) {
            if comment.span.end.offset <= end_offset {
                self.next_comment_index += 1;
            } else {
                break;
            }
        }
    }

    /// Checks if there's an ignore-next marker before this statement and consumes it.
    /// Returns the marker's start offset if the next statement should be ignored.
    pub(crate) fn consume_ignore_next_before(&mut self, stmt_start: u32) -> Option<u32> {
        if let Some(marker) = self.ignore_next_markers.get(self.next_ignore_next_index) {
            // The marker should be before this statement
            if marker.comment_end < stmt_start {
                self.next_ignore_next_index += 1;
                return Some(marker.comment_start);
            }
        }
        None
    }
}

impl HasFileId for FormatterState<'_, '_> {
    fn file_id(&self) -> FileId {
        self.file.id
    }
}

#[inline]
const fn is_insignificant(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b';' | b',')
}

#[inline]
const fn is_line_terminator(c: u8) -> bool {
    matches!(c, b'\n' | b'\r')
}

#[inline]
const fn is_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t')
}

#[inline]
const fn is_line_terminator_or_space(c: u8) -> bool {
    is_line_terminator(c) || is_space(c)
}
