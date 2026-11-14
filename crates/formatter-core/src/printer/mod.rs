use std::ops::Range;

use mago_allocator::Arena;
use mago_allocator::collections::{HashMap, HashSet};
use mago_allocator::vec::Vec;

use crate::document::Document;
use crate::document::group::GroupIdentifier;
use crate::printer::align::Mark;
use crate::printer::command::{Command, Indentation, Mode};
use crate::settings::PrinterSettings;

mod align;
mod breaks;
mod command;
mod fits;
mod group;
mod line;
mod print;

const MAP_PREALLOCATION_THRESHOLD: usize = 4 * 1024;
const GROUP_MODE_CAPACITY_DIVISOR: usize = 64;
const PROPAGATED_BREAK_CAPACITY_DIVISOR: usize = 16;

#[derive(Debug)]
pub struct Printer<'arena, A>
where
    A: Arena,
{
    arena: &'arena A,
    settings: PrinterSettings,
    out: Vec<'arena, u8, A>,
    output_start: usize,
    position: usize,
    commands: Vec<'arena, Command<'arena, A>, A>,
    line_suffix: Vec<'arena, Command<'arena, A>, A>,
    group_mode_map: HashMap<'arena, GroupIdentifier, Mode, A>,
    propagated_breaks: HashSet<'arena, usize, A>,
    new_line: &'static str,
    can_trim: bool,
    range_start: Option<usize>,
    range_end: Option<usize>,
    marks: Vec<'arena, Mark, A>,
    suffix_offsets: Vec<'arena, usize, A>,
}

impl<'arena, A: Arena> Printer<'arena, A> {
    pub fn new(arena: &'arena A, settings: PrinterSettings) -> Self {
        Self {
            arena,
            settings,
            out: Vec::new_in(arena),
            output_start: 0,
            position: 0,
            commands: Vec::new_in(arena),
            line_suffix: Vec::new_in(arena),
            group_mode_map: HashMap::new_in(arena),
            propagated_breaks: HashSet::new_in(arena),
            new_line: settings.end_of_line.as_str(),
            can_trim: true,
            range_start: None,
            range_end: None,
            marks: Vec::new_in(arena),
            suffix_offsets: Vec::new_in(arena),
        }
    }

    #[must_use]
    pub fn build(mut self, document: Document<'arena, A>, capacity_hint: usize) -> &'arena [u8] {
        self.reserve(capacity_hint);
        self.start_document(document, 0);
        self.print_commands();
        self.align();

        self.out.leak()
    }

    #[must_use]
    pub fn build_range(
        mut self,
        document: Document<'arena, A>,
        capacity_hint: usize,
    ) -> Option<(&'arena [u8], Range<usize>)> {
        self.reserve(capacity_hint);
        self.start_document(document, 0);
        self.print_commands();
        self.align();
        Some((self.out.leak(), self.range_start?..self.range_end?))
    }

    /// Append a document without changing the prefix, and reuse scratch buffers.
    pub fn print_into(
        &mut self,
        document: Document<'arena, A>,
        column: usize,
        output: &mut Vec<'arena, u8, A>,
    ) -> Range<usize> {
        self.start_document(document, column);
        std::mem::swap(&mut self.out, output);
        let start = self.out.len();
        self.output_start = start;
        self.print_commands();
        self.align();
        std::mem::swap(&mut self.out, output);

        start..output.len()
    }

    fn reserve(&mut self, capacity_hint: usize) {
        self.out.reserve_exact(capacity_hint);
        if capacity_hint >= MAP_PREALLOCATION_THRESHOLD {
            self.group_mode_map.reserve(capacity_hint / GROUP_MODE_CAPACITY_DIVISOR);
            self.propagated_breaks.reserve(capacity_hint / PROPAGATED_BREAK_CAPACITY_DIVISOR);
        }
    }

    fn start_document(&mut self, document: Document<'arena, A>, column: usize) {
        self.commands.clear();
        self.commands.push(Command::new(Indentation::root(), Mode::Break, self.arena.alloc(document)));
        self.line_suffix.clear();
        self.group_mode_map.clear();
        self.propagated_breaks.clear();
        self.position = column;
        self.output_start = 0;
        self.can_trim = true;
        self.range_start = None;
        self.range_end = None;
        self.marks.clear();
        self.suffix_offsets.clear();
    }
}
