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
    enabled: bool,
}

impl<'scratch, S> PHPDocResolution<'scratch, S>
where
    S: Arena,
{
    pub fn new(
        scratch: &'scratch S,
        trivia: &'scratch [Trivia<'scratch>],
        enabled: bool,
    ) -> PHPDocResolution<'scratch, S> {
        PHPDocResolution { scratch, trivia, enabled }
    }

    #[must_use]
    pub fn get(&self, node: Span) -> Option<Document<'scratch>> {
        if !self.enabled {
            return None;
        }

        let docblock = get_docblock_before_position(self.trivia, node.start_offset())?;

        Some(PHPDocParser::parse_with_span(self.scratch, docblock.value, docblock.span))
    }
}
