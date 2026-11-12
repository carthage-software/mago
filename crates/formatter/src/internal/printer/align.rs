use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_allocator::vec_in;

use crate::document::Column;
use crate::internal::printer::Printer;
use crate::internal::utils::string_width;

const MAX_CELLS: usize = 2;

#[derive(Debug)]
pub(super) struct Mark {
    column: Column,
    offset: usize,
    position: usize,
    line: usize,
}

#[derive(Clone, Copy, Default)]
struct Cell {
    offset: usize,
    position: usize,
    width: usize,
    gutter: usize,
}

struct Row {
    line: usize,
    adjacent: bool,
    aligned: bool,
    shape: u8,
    len: usize,
    cells: [Cell; MAX_CELLS],
    limit: usize,
}

#[derive(Clone, Copy)]
struct Run {
    widths: [usize; MAX_CELLS],
    total: usize,
    padded_limit: usize,
    unpadded_limit: usize,
    rows: usize,
}

impl Mark {
    pub(super) const fn new(column: Column, offset: usize, position: usize) -> Self {
        Self { column, offset, position, line: 0 }
    }
}

impl Row {
    fn anchor(&self) -> usize {
        self.cells[..self.len].iter().map(|cell| cell.width).sum()
    }
}

impl Run {
    const EMPTY: Self =
        Self { widths: [0; MAX_CELLS], total: 0, padded_limit: usize::MAX, unpadded_limit: usize::MAX, rows: 0 };

    /// Returns this run with `row` added, and whether every padded row still fits the print width.
    fn with(mut self, row: &Row) -> (Self, bool) {
        for (width, cell) in self.widths.iter_mut().zip(&row.cells[..row.len]) {
            *width = (*width).max(cell.width + cell.gutter);
        }

        let total = self.widths[..row.len].iter().sum();
        if total > self.total {
            self.padded_limit = self.padded_limit.min(self.unpadded_limit);
            self.unpadded_limit = usize::MAX;
            self.total = total;
        }

        if row.anchor() == total {
            self.unpadded_limit = self.unpadded_limit.min(row.limit);
        } else {
            self.padded_limit = self.padded_limit.min(row.limit);
        }

        self.rows += 1;

        (self, total <= self.padded_limit)
    }
}

impl<'arena, A> Printer<'arena, A>
where
    A: Arena,
{
    /// Pads the recorded alignment points into columns.
    ///
    /// This runs once the layout is final and only inserts spaces, so alignment can never change
    /// where a line breaks, and a line is only padded while it still fits the print width.
    pub(super) fn align(&mut self) {
        if self.marks.is_empty() {
            return;
        }

        let newline = self.new_line.as_bytes()[self.new_line.len() - 1];
        let (mut line, mut scanned) = (0, 0);
        for mark in &mut self.marks {
            mark.offset = mark.offset.min(self.out.len());
            line += memchr::memchr_iter(newline, &self.out[scanned..mark.offset]).count();
            (mark.line, scanned) = (line, mark.offset);
        }

        self.marks.sort_by_key(|mark| mark.column.table);

        let mut insertions: Vec<'arena, (usize, usize), A> = vec_in![self.arena];
        let mut rows: Vec<'arena, Row, A> = vec_in![self.arena];
        for marks in self.marks.chunk_by(|a, b| a.column.table == b.column.table) {
            rows.clear();
            for mark in marks {
                if mark.column.index == 0 {
                    rows.push(Row {
                        line: mark.line,
                        adjacent: mark.column.adjacent,
                        aligned: true,
                        shape: 0,
                        len: 0,
                        cells: [Cell::default(); MAX_CELLS],
                        limit: 0,
                    });
                } else if let Some(row) = rows.last_mut() {
                    row.aligned &=
                        mark.line == row.line && (row.len < MAX_CELLS || mark.column.index == Column::ANCHOR);
                    if row.aligned && mark.column.index != Column::ANCHOR {
                        let anchor = mark.position + usize::from(mark.column.shift);

                        row.cells[row.len] = Cell {
                            offset: mark.offset,
                            position: mark.position,
                            width: anchor.saturating_sub(row.anchor()),
                            gutter: usize::from(mark.column.gutter),
                        };
                        row.shape |= 1 << mark.column.index.min(7);
                        row.len += 1;
                    }
                }
            }

            for index in 0..rows.len() {
                let shares_line = |other: Option<&Row>| other.is_some_and(|other| other.line == rows[index].line);
                let aligned = rows[index].len > 0
                    && !shares_line(index.checked_sub(1).and_then(|previous| rows.get(previous)))
                    && !shares_line(rows.get(index + 1));

                let row = &mut rows[index];
                row.aligned &= aligned;
                if row.aligned {
                    let code_width = self.code_width(row.cells[row.len - 1], newline);

                    row.limit = (self.settings.print_width + row.anchor())
                        .checked_sub(code_width)
                        .filter(|_| code_width <= self.settings.print_width)
                        .unwrap_or(usize::MAX);
                }
            }

            let mut run: Option<(Run, usize, usize)> = None;
            for (index, row) in rows.iter().enumerate().filter(|(_, row)| row.aligned) {
                if let Some((current, start, last)) = run {
                    let previous = &rows[last];
                    let (extended, fits) = current.with(row);
                    if row.shape == previous.shape && (!row.adjacent || row.line == previous.line + 1) && fits {
                        run = Some((extended, start, index));
                        continue;
                    }

                    pad(&rows[start..=last], &current, &mut insertions);
                }

                run = Some((Run::EMPTY.with(row).0, index, index));
            }

            if let Some((current, start, last)) = run {
                pad(&rows[start..=last], &current, &mut insertions);
            }
        }

        if insertions.is_empty() {
            return;
        }

        insertions.sort_unstable_by_key(|&(offset, _)| offset);

        let inserted_before = |target: usize| -> usize {
            insertions.iter().take_while(|&&(offset, _)| offset < target).map(|&(_, count)| count).sum()
        };
        self.range_start = self.range_start.map(|offset| offset + inserted_before(offset));
        self.range_end = self.range_end.map(|offset| offset + inserted_before(offset));

        let padding: usize = insertions.iter().map(|&(_, count)| count).sum();
        let mut out = Vec::with_capacity_in(self.out.len() + padding, self.arena);
        let mut copied = 0;
        for &(offset, count) in &insertions {
            out.extend_from_slice(&self.out[copied..offset]);
            out.resize(out.len() + count, b' ');
            copied = offset;
        }
        out.extend_from_slice(&self.out[copied..]);

        self.out = out;
    }

    /// Width of the line holding `cell`, without its trailing comments.
    fn code_width(&self, cell: Cell, newline: u8) -> usize {
        let line_end =
            memchr::memchr(newline, &self.out[cell.offset..]).map_or(self.out.len(), |end| cell.offset + end);
        let suffix = self.suffix_offsets.partition_point(|&offset| offset < cell.offset);
        let end = self.suffix_offsets.get(suffix).map_or(line_end, |&offset| offset.min(line_end));

        cell.position + string_width(self.out[cell.offset..end].trim_ascii_end())
    }
}

fn pad<A>(rows: &[Row], run: &Run, insertions: &mut Vec<'_, (usize, usize), A>)
where
    A: Arena,
{
    if run.rows < 2 {
        return;
    }

    for row in rows.iter().filter(|row| row.aligned) {
        for (cell, width) in row.cells[..row.len].iter().zip(run.widths) {
            if width > cell.width {
                insertions.push((cell.offset, width - cell.width));
            }
        }
    }
}
