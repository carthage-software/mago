use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use crate::document::Document;
use crate::internal::printer::command::Mode;

const INLINE_DEPTH: usize = 16;

/// The documents still to measure, one frame per list of siblings. A frame's flag marks documents
/// inside a strict-fit group measured in flat mode, where any forced break means the group does
/// not fit.
pub(super) struct FitsStack<'doc, 'arena, A: Arena> {
    inline: [(Mode, bool, &'doc [Document<'arena, A>]); INLINE_DEPTH],
    overflow: Vec<'arena, (Mode, bool, &'doc [Document<'arena, A>]), A>,
    depth: usize,
}

impl<'doc, 'arena, A: Arena> FitsStack<'doc, 'arena, A> {
    pub(super) fn new(arena: &'arena A) -> Self {
        Self { inline: [(Mode::Flat, false, &[]); INLINE_DEPTH], overflow: Vec::new_in(arena), depth: 0 }
    }

    pub(super) fn push(&mut self, mode: Mode, strict: bool, documents: &'doc [Document<'arena, A>]) {
        if documents.is_empty() {
            return;
        }

        if self.depth < INLINE_DEPTH {
            self.inline[self.depth] = (mode, strict, documents);
        } else {
            self.overflow.push((mode, strict, documents));
        }
        self.depth += 1;
    }

    pub(super) fn next(&mut self) -> Option<(Mode, bool, &'doc Document<'arena, A>)> {
        while self.depth != 0 {
            let frame =
                if self.depth <= INLINE_DEPTH { &mut self.inline[self.depth - 1] } else { self.overflow.last_mut()? };

            if let Some((document, remaining)) = frame.2.split_first() {
                frame.2 = remaining;
                return Some((frame.0, frame.1, document));
            }

            if self.depth > INLINE_DEPTH {
                self.overflow.pop();
            }
            self.depth -= 1;
        }

        None
    }
}
