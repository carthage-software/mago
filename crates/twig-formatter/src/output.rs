use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_formatter_core::utils::column_after;

use crate::alignment::indentation_width;

#[derive(Clone, Copy)]
pub(super) struct Position {
    pub column: usize,
    pub indentation: usize,
}

pub(super) struct Output<'arena, A: Arena> {
    pub bytes: Vec<'arena, u8, A>,
    scanned: usize,
    column: usize,
    indentation: usize,
    has_content: bool,
    tab_width: usize,
}

impl<'arena, A: Arena> Output<'arena, A> {
    pub fn new(arena: &'arena A, capacity: usize, tab_width: usize) -> Self {
        Self {
            bytes: Vec::with_capacity_in(capacity, arena),
            scanned: 0,
            column: 0,
            indentation: 0,
            has_content: false,
            tab_width,
        }
    }

    pub fn position(&mut self) -> Position {
        let mut start = self.scanned;
        if let Some(index) = memchr::memrchr2(b'\r', b'\n', &self.bytes[start..]) {
            start += index + 1;
            self.column = 0;
            self.indentation = 0;
            self.has_content = false;
        }

        let added = &self.bytes[start..];
        self.column = column_after(added, self.column, self.tab_width);
        if !self.has_content {
            let indent_end = added.iter().position(|byte| !matches!(byte, b' ' | b'\t')).unwrap_or(added.len());
            self.indentation += indentation_width(&added[..indent_end], self.tab_width);
            self.has_content = indent_end != added.len();
        }
        self.scanned = self.bytes.len();

        Position { column: self.column, indentation: self.indentation }
    }

    pub fn finish(self) -> &'arena [u8] {
        self.bytes.leak()
    }
}
