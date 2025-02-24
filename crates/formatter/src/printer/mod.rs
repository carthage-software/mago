use std::collections::VecDeque;

use ahash::HashMap;

use mago_source::Source;

use crate::document::Align;
use crate::document::Document;
use crate::document::Fill;
use crate::document::IfBreak;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::document::Trim;
use crate::document::group::GroupIdentifier;
use crate::printer::command::Command;
use crate::printer::command::Indentation;
use crate::printer::command::Mode;
use crate::printer::utils::get_string_width;
use crate::settings::FormatSettings;

mod command;
mod utils;

#[derive(Debug)]
pub struct Printer<'a> {
    settings: FormatSettings,
    out: Vec<u8>,
    position: usize,
    commands: Vec<Command<'a>>,
    line_suffix: Vec<Command<'a>>,
    group_mode_map: HashMap<GroupIdentifier, Mode>,
    new_line: &'static str,
}

impl<'a> Printer<'a> {
    pub fn new(document: Document<'a>, source: &Source, settings: FormatSettings) -> Self {
        // Preallocate for performance because the output will very likely
        // be the same size as the original text.
        let out = Vec::with_capacity(source.size);
        let cmds = vec![Command::new(Indentation::root(), Mode::Break, document)];

        Self {
            settings,
            out,
            position: 0,
            commands: cmds,
            line_suffix: vec![],
            group_mode_map: HashMap::default(),
            new_line: settings.end_of_line.as_str(),
        }
    }

    pub fn build(mut self) -> String {
        self.print_doc_to_string();

        // SAFETY: We should have constructed valid UTF8 strings
        unsafe { String::from_utf8_unchecked(self.out) }
    }

    /// Turn Doc into a string
    pub fn print_doc_to_string(&mut self) {
        let mut should_remeasure = false;
        while let Some(Command { indentation, mut document, mode }) = self.commands.pop() {
            Self::propagate_breaks(&mut document);

            match document {
                Document::String(s) => self.handle_str(s),
                Document::Array(docs) => self.handle_array(indentation, mode, docs),
                Document::Indent(docs) => self.handle_indent(indentation, mode, docs),
                Document::Align(align) => self.handle_align(align, mode),
                Document::Group(_) => {
                    should_remeasure = self.handle_group(indentation, mode, document, should_remeasure);
                }
                Document::IndentIfBreak(docs) => self.handle_indent_if_break(indentation, mode, docs),
                Document::Line(line) => {
                    should_remeasure = self.handle_line(line, indentation, mode, document, should_remeasure);
                }
                Document::LineSuffix(docs) => self.handle_line_suffix(indentation, mode, docs),
                Document::LineSuffixBoundary => self.handle_line_suffix_boundary(indentation, mode),
                Document::IfBreak(if_break) => self.handle_if_break(if_break, indentation, mode),
                Document::Fill(fill) => self.handle_fill(indentation, mode, fill),
                Document::BreakParent => { /* No op */ }
                Document::Trim(trim) => self.handle_trim(trim),
            }

            if self.commands.is_empty() && !self.line_suffix.is_empty() {
                self.commands.extend(self.line_suffix.drain(..).rev());
            }
        }
    }

    fn remaining_width(&self) -> isize {
        (self.settings.print_width as isize) - (self.position as isize)
    }

    fn handle_str(&mut self, s: &str) {
        self.out.extend(s.as_bytes());
        self.position += get_string_width(s);
    }

    fn handle_array(&mut self, indentation: Indentation<'a>, mode: Mode, docs: Vec<Document<'a>>) {
        self.commands.extend(docs.into_iter().rev().map(|doc| Command::new(indentation.clone(), mode, doc)));
    }

    #[inline]
    fn handle_trim(&mut self, trim: Trim) {
        match trim {
            Trim::Whitespace => {
                while let Some(&last) = self.out.last() {
                    if last == b' ' || last == b'\t' {
                        self.out.pop();
                    } else {
                        break;
                    }
                }
            }
            Trim::Newlines => {
                while let Some(&last) = self.out.last() {
                    if last == b' ' || last == b'\t' || last == b'\n' {
                        self.out.pop();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn handle_indent(&mut self, indentation: Indentation<'a>, mode: Mode, docs: Vec<Document<'a>>) {
        let new_indentation = Indentation::Combined(vec![Indentation::Indent, indentation]);
        self.commands.extend(docs.into_iter().rev().map(|doc| Command::new(new_indentation.clone(), mode, doc)));
    }

    fn handle_align(&mut self, align: Align<'a>, mode: Mode) {
        let new_indent = Indentation::Alignment(align.alignment);
        self.commands.extend(align.contents.into_iter().rev().map(|doc| Command::new(new_indent.clone(), mode, doc)));
    }

    fn handle_group(
        &mut self,
        indentation: Indentation<'a>,
        mode: Mode,
        doc: Document<'a>,
        mut should_remeasure: bool,
    ) -> bool {
        let Document::Group(group) = doc else {
            unreachable!();
        };

        let should_break = group.should_break;
        let group_id = group.id;

        if mode.is_flat() && !should_remeasure {
            self.commands.extend(
                group
                    .contents
                    .into_iter()
                    .rev()
                    .map(|doc| Command::new(indentation.clone(), if should_break { Mode::Break } else { mode }, doc)),
            );

            self.set_group_mode_from_last_cmd(group_id);

            return should_remeasure;
        }

        should_remeasure = false;
        let remaining_width = self.remaining_width();
        let cmd = Command::new(indentation.clone(), Mode::Flat, Document::Group(group));
        if !should_break && self.fits(&cmd, remaining_width) {
            self.commands.push(Command::new(indentation.clone(), Mode::Flat, cmd.document));
        } else {
            let Document::Group(group) = cmd.document else {
                unreachable!();
            };

            if let Some(mut expanded_states) = group.expanded_states {
                let most_expanded = expanded_states.pop().unwrap();
                if should_break {
                    self.commands.push(Command::new(indentation, Mode::Break, most_expanded));

                    return should_remeasure;
                }

                for state in expanded_states {
                    let cmd = Command::new(indentation.clone(), Mode::Flat, state);
                    if self.fits(&cmd, remaining_width) {
                        self.commands.push(cmd);

                        return should_remeasure;
                    }
                }

                self.commands.push(Command::new(indentation, Mode::Break, most_expanded));
            } else {
                self.commands.push(Command::new(indentation, Mode::Break, Document::Array(group.contents)));
            }
        }

        self.set_group_mode_from_last_cmd(group_id);

        should_remeasure
    }

    fn handle_indent_if_break(&mut self, indentation: Indentation<'a>, mode: Mode, doc: IndentIfBreak<'a>) {
        let IndentIfBreak { contents, group_id } = doc;
        let group_mode = group_id.map_or(Some(mode), |id| self.group_mode_map.get(&id).copied());

        match group_mode {
            Some(Mode::Flat) => {
                self.commands
                    .extend(contents.into_iter().rev().map(|doc| Command::new(indentation.clone(), mode, doc)));
            }
            Some(Mode::Break) => {
                self.commands
                    .extend(contents.into_iter().rev().map(|doc| Command::new(Indentation::Indent, mode, doc)));
            }
            None => {}
        }
    }

    fn handle_line(
        &mut self,
        line: Line,
        indentation: Indentation<'a>,
        mode: Mode,
        doc: Document<'a>,
        mut should_remeasure: bool,
    ) -> bool {
        if mode.is_flat() {
            if !line.hard {
                if !line.soft {
                    self.out.push(b' ');
                    self.position += 1;
                }

                return should_remeasure;
            } else {
                should_remeasure = true;
            }
        }

        if !self.line_suffix.is_empty() {
            self.commands.push(Command::new(indentation, mode, doc));
            self.commands.extend(self.line_suffix.drain(..).rev());

            return should_remeasure;
        }

        if line.literal {
            self.out.extend(self.new_line.as_bytes());
            if !indentation.is_root() {
                self.position = 0;
            } else {
                self.position = self.add_indentation(indentation);
            }

            return should_remeasure;
        }

        self.handle_trim(Trim::Whitespace);
        self.out.extend(self.new_line.as_bytes());
        self.position = self.add_indentation(indentation);

        should_remeasure
    }

    fn handle_line_suffix(&mut self, indentation: Indentation<'a>, mode: Mode, docs: Vec<Document<'a>>) {
        self.line_suffix.push(Command { indentation, mode, document: Document::Array(docs) });
    }

    fn handle_line_suffix_boundary(&mut self, indentation: Indentation<'a>, mode: Mode) {
        if !self.line_suffix.is_empty() {
            self.commands.push(Command {
                indentation,
                mode,
                document: Document::Line(Line { hard: true, ..Line::default() }),
            });
        }
    }

    fn handle_if_break(&mut self, if_break: IfBreak<'a>, indentation: Indentation<'a>, mode: Mode) {
        let IfBreak { break_contents, flat_content, group_id } = if_break;
        let Some(group_mode) = group_id.map_or(Some(mode), |id| self.group_mode_map.get(&id).copied()) else {
            return;
        };

        match group_mode {
            Mode::Flat => {
                self.commands.push(Command::new(indentation, Mode::Flat, *flat_content));
            }
            Mode::Break => {
                self.commands.push(Command::new(indentation, Mode::Break, *break_contents));
            }
        }
    }

    fn handle_fill(&mut self, indentation: Indentation<'a>, mode: Mode, fill: Fill<'a>) {
        let mut fill = fill;
        let remaining_width = self.remaining_width();
        let original_parts_len = fill.parts().len();
        let (content, whitespace) = fill.drain_out_pair();

        let Some(content) = content else {
            return;
        };

        let content_flat_cmd = Command::new(indentation.clone(), Mode::Flat, content);
        let content_fits = self.fits(&content_flat_cmd, remaining_width);

        if original_parts_len == 1 {
            if content_fits {
                self.commands.push(content_flat_cmd);
            } else {
                self.commands.push(content_flat_cmd.with_mode(Mode::Break));
            }

            return;
        }

        let Some(whitespace) = whitespace else {
            return;
        };

        let whitespace_flat_cmd = Command::new(indentation.clone(), Mode::Flat, whitespace);
        if original_parts_len == 2 {
            if content_fits {
                self.commands.push(whitespace_flat_cmd);
                self.commands.push(content_flat_cmd);
            } else {
                let content_break_cmd = content_flat_cmd.with_mode(Mode::Break);
                let whitespace_break_cmd = whitespace_flat_cmd.with_mode(Mode::Break);
                self.commands.push(whitespace_break_cmd);
                self.commands.push(content_break_cmd);
            }

            return;
        }

        let Some(second_content) = fill.dequeue() else {
            return;
        };

        let mut docs = vec![];
        let content = content_flat_cmd.document;
        docs.push(content);
        docs.push(whitespace_flat_cmd.document);
        docs.push(second_content);

        let first_and_second_content_fit_cmd = Command::new(indentation.clone(), Mode::Flat, Document::Array(docs));
        let first_and_second_content_fits = self.fits(&first_and_second_content_fit_cmd, remaining_width);
        let Document::Array(mut doc) = first_and_second_content_fit_cmd.document else {
            return;
        };

        if let Some(second_content) = doc.pop() {
            fill.enqueue(second_content);
        }

        let Some(whitespace) = doc.pop() else {
            return;
        };
        let Some(content) = doc.pop() else {
            return;
        };

        let remaining_cmd = Command::new(indentation.clone(), mode, Document::Fill(fill));
        let whitespace_flat_cmd = Command::new(indentation.clone(), Mode::Flat, whitespace);
        let content_flat_cmd = Command::new(indentation, Mode::Flat, content);

        if first_and_second_content_fits {
            self.commands.extend(vec![remaining_cmd, whitespace_flat_cmd, content_flat_cmd]);
        } else if content_fits {
            self.commands.extend(vec![remaining_cmd, whitespace_flat_cmd.with_mode(Mode::Break), content_flat_cmd]);
        } else {
            self.commands.extend(vec![
                remaining_cmd,
                whitespace_flat_cmd.with_mode(Mode::Break),
                content_flat_cmd.with_mode(Mode::Break),
            ]);
        };
    }

    fn add_indentation(&mut self, indentation: Indentation) -> usize {
        let value = indentation.get_value(self.settings.use_tabs, self.settings.tab_width);
        self.out.extend(value.as_bytes());
        value.len()
    }

    fn set_group_mode_from_last_cmd(&mut self, id: Option<GroupIdentifier>) {
        let Some(id) = id else {
            return;
        };

        let Some(mode) = self.commands.last().map(|cmd| cmd.mode) else {
            return;
        };

        self.group_mode_map.insert(id, mode);
    }

    fn fits(&self, next: &Command<'a>, width: isize) -> bool {
        let mut remaining_width = width;
        let mut queue: VecDeque<(Mode, &Document)> = VecDeque::new();
        let mut cmds = self.commands.iter().rev();

        queue.push_front((next.mode, &next.document));
        while let Some((mode, doc)) = queue.pop_front() {
            match doc {
                Document::String(string) => {
                    remaining_width -= get_string_width(string) as isize;
                }
                Document::IndentIfBreak(IndentIfBreak { contents, .. })
                | Document::Indent(contents)
                | Document::Align(Align { contents, .. })
                | Document::Array(contents) => {
                    for d in contents.iter().rev() {
                        queue.push_front((mode, d));
                    }
                }
                Document::Group(group) => {
                    let mode = if group.should_break { Mode::Break } else { mode };
                    if group.expanded_states.is_some() && mode.is_break() {
                        queue.push_front((mode, group.expanded_states.as_ref().unwrap().last().unwrap()));
                    } else {
                        for d in group.contents.iter().rev() {
                            queue.push_front((mode, d));
                        }
                    };
                }
                Document::IfBreak(if_break_doc) => {
                    let group_mode =
                        if_break_doc.group_id.map_or(mode, |id| *self.group_mode_map.get(&id).unwrap_or(&Mode::Flat));

                    let contents =
                        if group_mode.is_break() { &if_break_doc.break_contents } else { &if_break_doc.flat_content };

                    queue.push_front((mode, contents));
                }
                Document::Line(line) => {
                    if mode.is_break() || line.hard {
                        return true;
                    }
                    if !line.soft {
                        remaining_width -= 1_isize;
                    }
                }
                Document::Fill(fill) => {
                    for part in fill.parts().iter().rev() {
                        queue.push_front((mode, part));
                    }
                }
                Document::LineSuffix(_) => {
                    break;
                }
                Document::LineSuffixBoundary => {
                    if !self.line_suffix.is_empty() {
                        return false;
                    }

                    break;
                }
                Document::BreakParent => {}
                Document::Trim(_) => {}
            }

            if remaining_width < 0 {
                return false;
            }

            if queue.is_empty() {
                if let Some(cmd) = cmds.next() {
                    queue.push_back((cmd.mode, &cmd.document));
                }
            }
        }

        true
    }

    pub fn propagate_breaks(doc: &mut Document<'_>) -> bool {
        let check_array = |arr: &mut Vec<Document<'_>>| arr.iter_mut().rev().any(|doc| Self::propagate_breaks(doc));

        match doc {
            Document::BreakParent => true,
            Document::Group(group) => {
                let mut should_break = false;
                if let Some(expanded_states) = &mut group.expanded_states {
                    should_break = expanded_states.iter_mut().rev().any(Self::propagate_breaks);
                }
                if !should_break {
                    should_break = check_array(&mut group.contents);
                }
                if group.expanded_states.is_none() && should_break {
                    group.should_break = should_break;
                }
                group.should_break
            }
            Document::IfBreak(d) => Self::propagate_breaks(&mut d.break_contents),
            Document::Array(arr)
            | Document::Indent(arr)
            | Document::Align(Align { contents: arr, .. })
            | Document::IndentIfBreak(IndentIfBreak { contents: arr, .. }) => check_array(arr),
            _ => false,
        }
    }
}
