use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use crate::document::Document;
use crate::internal::printer::command::Mode;

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
