use mago_allocator::Arena;

use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::cst::Document;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::comments::docblock::get_docblock_before_position;
use mago_syntax::cst::Trivia;

#[derive(Debug)]
pub struct PHPDocResolution<'scratch, S>
where
    S: Arena,
{
    scratch: &'scratch S,
    trivia: &'scratch [Trivia<'scratch>],
}

impl<'scratch, S> PHPDocResolution<'scratch, S>
where
    S: Arena,
{
    pub fn new(scratch: &'scratch S, trivia: &'scratch [Trivia<'scratch>]) -> PHPDocResolution<'scratch, S> {
        PHPDocResolution { scratch, trivia }
    }

    #[must_use]
    pub fn get(&self, node: Span) -> Option<Document<'scratch>> {
        let docblock = get_docblock_before_position(self.trivia, node.start_offset())?;

        Some(PHPDocParser::parse_with_span(self.scratch, docblock.value, docblock.span))
    }
}
