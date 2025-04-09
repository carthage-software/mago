use std::iter::Peekable;
use std::vec::IntoIter;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_php_version::PHPVersion;
use mago_source::Source;
use mago_span::Span;
use mago_syntax::ast::Node;
use mago_syntax::ast::Trivia;

use crate::document::group::GroupIdentifier;
use crate::document::group::GroupIdentifierBuilder;
use crate::settings::FormatSettings;

pub mod binaryish;
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

#[derive(Debug)]
pub struct FormatterState<'a> {
    interner: &'a ThreadedInterner,
    source: &'a Source,
    source_text: &'a str,
    php_version: PHPVersion,
    settings: FormatSettings,
    stack: Vec<Node<'a>>,
    comments: Peekable<IntoIter<Trivia>>,
    scripting_mode: bool,
    id_builder: GroupIdentifierBuilder,
    argument_state: ArgumentState,
    parameter_state: ParameterState,
}

impl<'a> FormatterState<'a> {
    pub fn new(
        interner: &'a ThreadedInterner,
        source: &'a Source,
        php_version: PHPVersion,
        settings: FormatSettings,
    ) -> Self {
        Self {
            interner,
            source,
            source_text: interner.lookup(&source.content),
            php_version,
            settings,
            stack: vec![],
            comments: vec![].into_iter().peekable(),
            scripting_mode: false,
            id_builder: GroupIdentifierBuilder::new(),
            argument_state: ArgumentState::default(),
            parameter_state: ParameterState::default(),
        }
    }

    fn next_id(&mut self) -> GroupIdentifier {
        self.id_builder.next_id()
    }

    fn lookup(&self, string: &StringIdentifier) -> &'a str {
        self.interner.lookup(string)
    }

    #[inline]
    fn as_str(&self, string: impl AsRef<str>) -> &'a str {
        self.interner.interned_str(string)
    }

    #[inline]
    fn enter_node(&mut self, node: Node<'a>) {
        self.stack.push(node);
    }

    #[inline]
    fn leave_node(&mut self) {
        self.stack.pop();
    }

    #[inline]
    fn current_node(&self) -> Node<'a> {
        self.stack[self.stack.len() - 1]
    }

    #[inline]
    fn parent_node(&self) -> Node<'a> {
        self.stack[self.stack.len() - 2]
    }

    #[inline]
    fn grandparent_node(&self) -> Option<Node<'a>> {
        let len = self.stack.len();

        (len > 2).then(|| self.stack[len - 2 - 1])
    }

    #[inline]
    fn great_grandparent_node(&self) -> Option<Node<'a>> {
        let len = self.stack.len();
        (len > 3).then(|| self.stack[len - 3 - 1])
    }

    #[inline]
    fn nth_parent_kind(&self, n: usize) -> Option<Node<'a>> {
        let len = self.stack.len();

        (len > n).then(|| self.stack[len - n - 1])
    }

    #[inline]
    fn is_previous_line_empty(&self, start_index: usize) -> bool {
        let idx = start_index - 1;
        let idx = self.skip_spaces(Some(idx), true);
        let idx = self.skip_newline(idx, true);
        let idx = self.skip_spaces(idx, true);
        let idx2 = self.skip_newline(idx, true);
        idx != idx2
    }

    #[inline]
    fn is_next_line_empty(&self, span: Span) -> bool {
        self.is_next_line_empty_after_index(span.end.offset)
    }

    #[inline]
    fn is_next_line_empty_after_index(&self, start_index: usize) -> bool {
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
    fn skip_inline_comments(&self, start_index: Option<usize>) -> Option<usize> {
        let start_index = start_index?;
        if start_index + 1 >= self.source_text.len() {
            return Some(start_index); // Not enough characters to check for comment
        }

        if self.source_text[start_index..].starts_with("//")
            || (self.source_text[start_index..].starts_with("#")
                && !self.source_text[start_index + 1..].starts_with("["))
        {
            return self.skip_everything_but_new_line(Some(start_index), false);
        }

        if self.source_text[start_index..].starts_with("/*") {
            // Find the closing */
            if let Some(end_pos) = self.source_text[start_index + 2..].find("*/") {
                let end_index = start_index + 2 + end_pos + 2; // +2 for the "*/" itself

                // Check if there's a newline between /* and */
                let comment_text = &self.source_text[start_index..end_index];
                if !comment_text.contains('\n') && !comment_text.contains('\r') {
                    return Some(end_index);
                }

                // If there's a newline, we don't consider it an inline comment
                // so we don't skip it
            }
        }

        Some(start_index)
    }

    #[inline]
    fn skip_to_line_end(&self, start_index: Option<usize>) -> Option<usize> {
        let mut index = self.skip(start_index, false, |c| matches!(c, b' ' | b'\t' | b',' | b';'));
        index = self.skip_inline_comments(index);
        index
    }

    #[inline]
    fn skip_spaces(&self, start_index: Option<usize>, backwards: bool) -> Option<usize> {
        self.skip(start_index, backwards, |c| matches!(c, b' ' | b'\t'))
    }

    #[inline]
    fn skip_spaces_and_new_lines(&self, start_index: Option<usize>, backwards: bool) -> Option<usize> {
        self.skip(start_index, backwards, |c| matches!(c, b' ' | b'\t' | b'\r' | b'\n'))
    }

    #[inline]
    fn skip_everything_but_new_line(&self, start_index: Option<usize>, backwards: bool) -> Option<usize> {
        self.skip(start_index, backwards, |c| !matches!(c, b'\r' | b'\n'))
    }

    #[inline]
    fn skip<F>(&self, start_index: Option<usize>, backwards: bool, f: F) -> Option<usize>
    where
        F: Fn(u8) -> bool,
    {
        let start_index = start_index?;
        let mut index = start_index;
        if backwards {
            for c in self.source_text[..=start_index].bytes().rev() {
                if !f(c) {
                    return Some(index);
                }
                index -= 1;
            }
        } else {
            let source_bytes = self.source_text.as_bytes();
            let text_len = source_bytes.len();
            while index < text_len {
                if !f(source_bytes[index]) {
                    return Some(index);
                }
                index += 1;
            }
        }

        None
    }

    #[inline]
    fn skip_newline(&self, start_index: Option<usize>, backwards: bool) -> Option<usize> {
        let start_index = start_index?;
        let c = if backwards {
            self.source_text[..=start_index].bytes().next_back()
        } else {
            self.source_text[start_index..].bytes().next()
        }?;

        if matches!(c, b'\n') {
            return Some(if backwards { start_index - 1 } else { start_index + 1 });
        }

        if matches!(c, b'\r') {
            let next_index = if backwards { start_index - 1 } else { start_index + 1 };
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
    fn has_newline(&self, start_index: usize, backwards: bool) -> bool {
        if (backwards && start_index == 0) || (!backwards && start_index == self.source_text.len()) {
            return false;
        }
        let start_index = if backwards { start_index - 1 } else { start_index };
        let idx = self.skip_spaces(Some(start_index), backwards);
        let idx2 = self.skip_newline(idx, backwards);
        idx != idx2
    }

    #[inline]
    fn split_lines(slice: &'a str) -> Vec<&'a str> {
        let mut lines = Vec::new();
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
    fn skip_leading_whitespace_up_to(s: &'a str, indent: usize) -> &'a str {
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
}
