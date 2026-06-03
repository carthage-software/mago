use bumpalo::Bump;

use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::cst::Document;
use mago_span::{HasSpan, Span};
use mago_syntax::comments::docblock::get_docblock_before_position;
use mago_syntax::cst::Trivia;

#[derive(Debug)]
pub struct PHPDocResolution<'arena> {
    arena: &'arena Bump,
    trivia: &'arena [Trivia<'arena>],
}

impl<'arena> PHPDocResolution<'arena> {
    pub fn new(arena: &'arena Bump, trivia: &'arena [Trivia<'arena>]) -> PHPDocResolution<'arena> {
        PHPDocResolution { arena, trivia }
    }

    #[must_use]
    pub fn get(&self, node: Span) -> Option<Document<'arena>> {
        let docblock = get_docblock_before_position(self.trivia, node.start_offset())?;

        Some(PHPDocParser::parse_with_span(self.arena, docblock.value, docblock.span))
    }
}
