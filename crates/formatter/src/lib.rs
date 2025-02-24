use std::iter::Peekable;
use std::vec::IntoIter;

use mago_ast::Node;
use mago_ast::Program;
use mago_ast::Trivia;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_source::Source;
use mago_span::Span;

use crate::document::Document;
use crate::document::group::GroupIdentifier;
use crate::document::group::GroupIdentifierBuilder;
use crate::format::Format;
use crate::printer::Printer;
use crate::settings::FormatSettings;

pub mod settings;

mod binaryish;
mod comment;
mod document;
mod format;
mod macros;
mod parens;
mod printer;
mod utils;

/// Format the given program.
///
/// # Arguments
///
/// * `interner` - The interner to use for string interning.
/// * `source` - The source to use for the program.
/// * `program` - A `Program` struct representing the AST of the program to format.
/// * `settings` - A `FormatSettings` struct that contains the settings to use for formatting.
///
/// # Returns
///
/// The formatted program as a string.
pub fn format<'a>(
    interner: &'a ThreadedInterner,
    source: &'a Source,
    program: &'a Program,
    settings: FormatSettings,
) -> String {
    let mut formatter = Formatter::new(interner, source, settings);

    formatter.format(program)
}

#[derive(Debug)]
struct ArgumentState {
    expand_first_argument: bool,
    expand_last_argument: bool,
}

#[derive(Debug)]
pub struct Formatter<'a> {
    interner: &'a ThreadedInterner,
    source: &'a Source,
    source_text: &'a str,
    settings: FormatSettings,
    stack: Vec<Node<'a>>,
    comments: Peekable<IntoIter<Trivia>>,
    scripting_mode: bool,
    id_builder: GroupIdentifierBuilder,
    argument_state: ArgumentState,
}

impl<'a> Formatter<'a> {
    pub fn new(interner: &'a ThreadedInterner, source: &'a Source, settings: FormatSettings) -> Self {
        Self {
            interner,
            source,
            source_text: interner.lookup(&source.content),
            settings,
            stack: vec![],
            comments: vec![].into_iter().peekable(),
            scripting_mode: false,
            id_builder: GroupIdentifierBuilder::new(),
            argument_state: ArgumentState { expand_first_argument: false, expand_last_argument: false },
        }
    }

    pub fn format(&mut self, program: &'a Program) -> String {
        let document = self.build(program);

        let printer = Printer::new(document, self.source, self.settings);

        printer.build()
    }

    fn build(&mut self, program: &'a Program) -> Document<'a> {
        self.comments =
            program.trivia.iter().filter(|t| t.kind.is_comment()).copied().collect::<Vec<_>>().into_iter().peekable();

        program.format(self)
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
            idx = self.skip_single_line_comments(idx);
            idx = self.skip_spaces(idx, /* backwards */ false);
        }

        idx = self.skip_single_line_comments(idx);
        idx = self.skip_newline(idx, /* backwards */ false);
        idx.is_some_and(|idx| self.has_newline(idx, /* backwards */ false))
    }

    #[inline]
    fn skip_single_line_comments(&self, start_index: Option<usize>) -> Option<usize> {
        let start_index = start_index?;
        if start_index + 1 >= self.source_text.len() {
            return Some(start_index); // Not enough characters to check for comment
        }

        if self.source_text[start_index..].starts_with("//")
            || (self.source_text[start_index..].starts_with("#")
                && !self.source_text[start_index + 1..].starts_with("["))
        {
            self.skip_everything_but_new_line(Some(start_index), false)
        } else {
            Some(start_index)
        }
    }

    #[inline]
    fn skip_to_line_end(&self, start_index: Option<usize>) -> Option<usize> {
        let mut index = self.skip(start_index, false, |c| matches!(c, b' ' | b'\t' | b',' | b';'));
        index = self.skip_single_line_comments(index);
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
        slice.split_inclusive('\n').map(|line| line.trim_end_matches('\n').trim_end_matches('\r')).collect()
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
