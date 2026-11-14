use crate::document::{Document, Line, Space, Trim};
use crate::printer::Printer;
use crate::printer::command::{Command, Indentation, Mode};
use crate::utils::{column_after, is_line_terminator_or_space, is_space};
use mago_allocator::Arena;

impl<'arena, A: Arena> Printer<'arena, A> {
    pub(super) fn handle_bytes(&mut self, s: &'arena [u8]) {
        self.out.extend(s);
        self.position = column_after(s, self.position, self.settings.tab_width);
    }

    pub(super) fn handle_space(&mut self, s: Space) {
        if s.soft {
            if let Some(&last) = self.out.last()
                && is_line_terminator_or_space(last)
            {
                return;
            }
        }

        self.out.push(32u8);
        self.position += 1;
    }

    #[inline]
    pub(super) fn handle_trim(&mut self, trim: Trim) {
        if !self.can_trim {
            self.can_trim = true;

            return;
        }

        let should_pop: fn(u8) -> bool = match trim {
            Trim::Whitespace => is_space,
            Trim::Newlines => is_line_terminator_or_space,
        };

        while let Some(&last) = self.out.last() {
            if self.out.len() <= self.output_start || !should_pop(last) {
                break;
            }

            self.out.pop();
        }
        self.range_start = self.range_start.map(|offset| offset.min(self.out.len()));
        self.range_end = self.range_end.map(|offset| offset.min(self.out.len()));
    }

    pub(super) fn handle_line(
        &mut self,
        line: Line,
        indentation: &Indentation<'arena>,
        mode: Mode,
        doc: &'arena Document<'arena, A>,
        mut should_remeasure: bool,
    ) -> bool {
        if mode.is_flat() {
            if !line.hard {
                if !line.soft {
                    self.out.push(b' ');
                    self.position += 1;
                }

                return should_remeasure;
            }

            should_remeasure = true;
        }

        if !self.line_suffix.is_empty() {
            self.commands.push(Command::new(*indentation, mode, doc));
            self.flush_line_suffix();

            return should_remeasure;
        }

        if line.literal {
            self.out.extend(self.new_line.as_bytes());
            if indentation.is_root() {
                self.position = self.add_indentation(indentation);
            } else {
                self.position = 0;
            }

            return should_remeasure;
        }

        self.handle_trim(Trim::Whitespace);
        self.out.extend(self.new_line.as_bytes());
        self.position = self.add_indentation(indentation);

        should_remeasure
    }

    pub(super) fn handle_line_suffix_boundary(
        &mut self,
        indentation: Indentation<'arena>,
        mode: Mode,
        mut should_remeasure: bool,
    ) -> bool {
        if !self.line_suffix.is_empty() {
            self.commands.push(Command::new(indentation, mode, self.arena.alloc(Document::space())));
            self.flush_line_suffix();

            should_remeasure = true;
        }

        should_remeasure
    }

    pub(super) fn add_indentation(&mut self, indentation: &Indentation<'arena>) -> usize {
        let depth = indentation.depth();
        let indentation_width = depth * self.settings.tab_width;
        let indentation_byte = if self.settings.use_tabs { b'\t' } else { b' ' };
        let indentation_length = if self.settings.use_tabs { depth } else { indentation_width };
        self.out.resize(self.out.len() + indentation_length, indentation_byte);

        if let Some(alignment) = indentation.alignment() {
            self.out.extend(alignment);
            column_after(alignment, indentation_width, self.settings.tab_width)
        } else {
            indentation_width
        }
    }
}
