use std::iter::Peekable;
use std::vec::IntoIter;

use fennec_ast::Node;
use fennec_ast::Program;
use fennec_ast::Trivia;
use fennec_interner::StringIdentifier;
use fennec_interner::ThreadedInterner;
use fennec_source::Source;
use fennec_span::Span;

use crate::document::group::GroupIdentifier;
use crate::document::group::GroupIdentifierBuilder;
use crate::document::Document;
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

pub fn format<'a>(
    settings: FormatSettings,
    interner: &'a ThreadedInterner,
    source: &'a Source,
    program: &'a Program,
) -> String {
    let mut formatter = Formatter::new(interner, source, settings);
    let document = formatter.format(program);

    let printer = Printer::new(document, &formatter.source, formatter.settings);

    printer.build()
}

struct ArgumentState {
    expand_first_argument: bool,
    expand_last_argument: bool,
}

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

    pub fn format(&mut self, program: &'a Program) -> Document<'a> {
        self.comments =
            program.trivia.iter().filter(|t| t.kind.is_comment()).copied().collect::<Vec<_>>().into_iter().peekable();

        program.format(self)
    }

    pub(crate) fn next_id(&mut self) -> GroupIdentifier {
        self.id_builder.next_id()
    }

    pub(crate) fn lookup(&self, string: &StringIdentifier) -> &'a str {
        self.interner.lookup(string)
    }

    pub(crate) fn as_str(&self, string: impl AsRef<str>) -> &'a str {
        self.interner.interned_str(string)
    }

    pub(crate) fn enter_node(&mut self, node: Node<'a>) {
        self.stack.push(node);
    }

    pub(crate) fn leave_node(&mut self) {
        self.stack.pop();
    }

    pub(crate) fn current_node(&self) -> Node<'a> {
        self.stack[self.stack.len() - 1]
    }

    pub(crate) fn parent_node(&self) -> Node<'a> {
        self.stack[self.stack.len() - 2]
    }

    pub(crate) fn grandparent_node(&self) -> Option<Node<'a>> {
        let len = self.stack.len();
        (len > 2).then(|| self.stack[len - 2 - 1])
    }

    pub(crate) fn nth_parent_kind(&self, n: usize) -> Option<Node<'a>> {
        let len = self.stack.len();
        (len > n).then(|| self.stack[len - n - 1])
    }

    fn is_previous_line_empty(&self, start_index: usize) -> bool {
        let idx = start_index - 1;
        let idx = self.skip_spaces(Some(idx), true);
        let idx = self.skip_newline(idx, true);
        let idx = self.skip_spaces(idx, true);
        let idx2 = self.skip_newline(idx, true);
        idx != idx2
    }

    pub(crate) fn is_next_line_empty(&self, span: Span) -> bool {
        self.is_next_line_empty_after_index(span.end.offset)
    }

    pub(crate) fn is_next_line_empty_after_index(&self, start_index: usize) -> bool {
        let mut old_idx = None;
        let mut idx = Some(start_index);
        while idx != old_idx {
            old_idx = idx;
            idx = self.skip_to_line_end(idx);
            idx = self.skip_inline_comment(idx);
            idx = self.skip_spaces(idx, /* backwards */ false);
        }

        idx = self.skip_trailing_comment(idx);
        idx = self.skip_newline(idx, /* backwards */ false);
        idx.is_some_and(|idx| self.has_newline(idx, /* backwards */ false))
    }

    pub(crate) fn skip_trailing_comment(&self, start_index: Option<usize>) -> Option<usize> {
        let start_index = start_index?;
        let mut chars = self.source_text[start_index as usize..].chars();
        let c = chars.next()?;
        if c != '/' {
            return Some(start_index);
        }

        let c = chars.next()?;
        if c != '/' {
            return Some(start_index);
        }

        self.skip_everything_but_new_line(Some(start_index), /* backwards */ false)
    }

    pub(crate) fn skip_inline_comment(&self, start_index: Option<usize>) -> Option<usize> {
        let start_index = start_index?;
        Some(start_index)
    }

    pub(crate) fn skip_to_line_end(&self, start_index: Option<usize>) -> Option<usize> {
        self.skip(start_index, false, |c| matches!(c, ' ' | '\t' | ',' | ';'))
    }

    pub(crate) fn skip_spaces(&self, start_index: Option<usize>, backwards: bool) -> Option<usize> {
        self.skip(start_index, backwards, |c| matches!(c, ' ' | '\t'))
    }

    pub(crate) fn skip_spaces_and_new_lines(&self, start_index: Option<usize>, backwards: bool) -> Option<usize> {
        self.skip(start_index, backwards, |c| matches!(c, ' ' | '\t' | '\r' | '\n') || is_line_terminator(c))
    }

    pub(crate) fn skip_everything_but_new_line(&self, start_index: Option<usize>, backwards: bool) -> Option<usize> {
        self.skip(start_index, backwards, |c| !is_line_terminator(c))
    }

    pub(crate) fn skip<F>(&self, start_index: Option<usize>, backwards: bool, f: F) -> Option<usize>
    where
        F: Fn(char) -> bool,
    {
        let start_index = start_index?;
        let mut index = start_index;
        if backwards {
            for c in self.source_text[..=start_index as usize].chars().rev() {
                if !f(c) {
                    return Some(index);
                }
                index -= 1;
            }
        } else {
            for c in self.source_text[start_index as usize..].chars() {
                if !f(c) {
                    return Some(index);
                }

                index += 1;
            }
        }

        None
    }

    pub(crate) fn skip_newline(&self, start_index: Option<usize>, backwards: bool) -> Option<usize> {
        let start_index = start_index?;
        let c = if backwards {
            self.source_text[..=start_index as usize].chars().next_back()
        } else {
            self.source_text[start_index as usize..].chars().next()
        }?;
        if is_line_terminator(c) {
            let len = c.len_utf8();
            return Some(if backwards { start_index - len } else { start_index + len });
        }
        Some(start_index)
    }

    pub(crate) fn has_newline(&self, start_index: usize, backwards: bool) -> bool {
        if (backwards && start_index == 0) || (!backwards && start_index as usize == self.source_text.len()) {
            return false;
        }
        let start_index = if backwards { start_index - 1 } else { start_index };
        let idx = self.skip_spaces(Some(start_index), backwards);
        let idx2 = self.skip_newline(idx, backwards);
        idx != idx2
    }

    pub(crate) fn split_lines(slice: &'a str) -> Vec<&'a str> {
        let bytes = slice.as_bytes();
        let mut lines = Vec::new();

        let mut start = 0;
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\n' => {
                    lines.push(&slice[start..i]);
                    start = i + 1;
                }
                b'\r' => {
                    lines.push(&slice[start..i]);
                    start = i + 1;
                    if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                        i += 1;
                    }
                }
                _ => {}
            }
            i += 1;
        }

        if start < bytes.len() {
            lines.push(&slice[start..]);
        }

        lines
    }

    pub(crate) fn skip_leading_whitespace_up_to(s: &'a str, indent: usize) -> &'a str {
        let mut count = 0;
        let mut position = 0;

        for (i, c) in s.char_indices() {
            if !c.is_whitespace() || count >= indent {
                break;
            }
            count += 1;
            position = i + c.len_utf8();
        }

        &s[position..]
    }
}

pub(crate) const fn is_line_terminator(c: char) -> bool {
    matches!(c, '\u{a}' | '\u{d}' | '\u{2028}' | '\u{2029}')
}
