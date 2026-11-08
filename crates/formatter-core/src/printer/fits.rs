use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use crate::document::{Align, BreakMode, Document, IndentIfBreak};
use crate::printer::Printer;
use crate::printer::command::Mode;
use crate::printer::command::{Command, CommandDocument};
use crate::utils::{column_after, is_ascii_line, is_space, string_width};

const INLINE_DEPTH: usize = 16;

pub(super) struct FitsStack<'doc, 'arena, A: Arena> {
    inline: [(Mode, &'doc [Document<'arena, A>]); INLINE_DEPTH],
    overflow: Vec<'arena, (Mode, &'doc [Document<'arena, A>]), A>,
    depth: usize,
}

impl<'doc, 'arena, A: Arena> FitsStack<'doc, 'arena, A> {
    pub(super) fn new(arena: &'arena A) -> Self {
        Self { inline: [(Mode::Flat, &[]); INLINE_DEPTH], overflow: Vec::new_in(arena), depth: 0 }
    }

    pub(super) fn push(&mut self, mode: Mode, documents: &'doc [Document<'arena, A>]) {
        if documents.is_empty() {
            return;
        }

        if self.depth < INLINE_DEPTH {
            self.inline[self.depth] = (mode, documents);
        } else {
            self.overflow.push((mode, documents));
        }
        self.depth += 1;
    }

    pub(super) fn next(&mut self) -> Option<(Mode, &'doc Document<'arena, A>)> {
        while self.depth != 0 {
            let frame =
                if self.depth <= INLINE_DEPTH { &mut self.inline[self.depth - 1] } else { self.overflow.last_mut()? };

            if let Some((document, remaining)) = frame.1.split_first() {
                let mode = frame.0;
                if remaining.is_empty() {
                    if self.depth > INLINE_DEPTH {
                        self.overflow.pop();
                    }

                    self.depth -= 1;
                } else {
                    frame.1 = remaining;
                }

                return Some((mode, document));
            }

            if self.depth > INLINE_DEPTH {
                self.overflow.pop();
            }
            self.depth -= 1;
        }

        None
    }
}

impl<'arena, A: Arena> Printer<'arena, A> {
    pub(super) fn fits(&self, next: &Command<'arena, A>, width: isize) -> bool {
        if width < 0 {
            return false;
        }

        let mut remaining_width = width;
        let mut has_line_suffix = false;
        let mut stack = FitsStack::new(self.arena);
        let mut cmds = self.commands.iter().rev();

        let mut first = Some(next);
        while let Some((mode, document)) =
            stack.next().map(|(mode, doc)| (mode, CommandDocument::Document(doc))).or_else(|| {
                first
                    .take()
                    .or_else(|| if has_line_suffix { None } else { cmds.next() })
                    .map(|cmd| (cmd.mode, cmd.document))
            })
        {
            let doc = match document {
                CommandDocument::Document(doc) => doc,
                CommandDocument::Array(docs) => {
                    if remaining_width < 0 {
                        return false;
                    }

                    stack.push(mode, docs);
                    continue;
                }
            };
            match doc {
                Document::String(string) => {
                    if is_ascii_line(string) {
                        remaining_width -= string.len() as isize;
                    } else if memchr::memchr2(b'\r', b'\n', string).is_some() {
                        remaining_width -= string_width(string) as isize;
                    } else {
                        let column = self.settings.print_width - remaining_width as usize;
                        remaining_width = self.settings.print_width as isize
                            - column_after(string, column, self.settings.tab_width) as isize;
                    }
                }
                Document::Space(space) => {
                    if !space.soft || self.out.last().is_none_or(|&b| !is_space(b)) {
                        remaining_width -= 1;
                    }
                }
                Document::IndentIfBreak(IndentIfBreak { contents, .. })
                | Document::Indent(contents)
                | Document::Align(Align { contents, .. })
                | Document::Array(contents) => {
                    stack.push(mode, contents);
                }
                Document::Group(group) => {
                    let group_break_mode = *group.break_mode.borrow();
                    if group_break_mode == BreakMode::Preserve && mode.is_flat() {
                        return false;
                    }

                    let group_mode = if group_break_mode == BreakMode::Force { Mode::Break } else { mode };
                    if group.expanded_states.is_some() && group_mode.is_break() {
                        if let Some(expanded_states) = group.expanded_states.as_ref()
                            && let Some(last_state) = expanded_states.last()
                        {
                            stack.push(group_mode, std::slice::from_ref(last_state));
                        }
                    } else {
                        stack.push(group_mode, &group.contents);
                    }
                }
                Document::IfBreak(if_break_doc) => {
                    let group_mode =
                        if_break_doc.group_id.map_or(mode, |id| *self.group_mode_map.get(&id).unwrap_or(&Mode::Flat));
                    let contents =
                        if group_mode.is_break() { if_break_doc.break_contents } else { if_break_doc.flat_content };
                    stack.push(mode, std::slice::from_ref(contents));
                }
                Document::Line(line) => {
                    if mode.is_break() || line.hard {
                        return true;
                    }
                    if !line.soft {
                        remaining_width -= 1;
                    }
                }
                Document::LineSuffix(_) => {
                    has_line_suffix = true;
                }
                Document::LineSuffixBoundary => {
                    if has_line_suffix || !self.line_suffix.is_empty() {
                        return false;
                    }
                }
                Document::BreakParent
                | Document::Trim(_)
                | Document::DoNotTrim
                | Document::RangeStart
                | Document::RangeEnd
                | Document::Column(_) => {}
            }

            if remaining_width < 0 {
                return false;
            }
        }

        true
    }
}
