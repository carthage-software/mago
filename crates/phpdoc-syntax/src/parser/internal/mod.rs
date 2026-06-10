use mago_allocator::Arena;
use mago_span::Span;
use mago_syntax_core::cst::Sequence;

use crate::cst::Document;
use crate::cst::element::Element;
use crate::parser::PHPDocParser;

pub(crate) mod alloc;
pub(crate) mod code;
pub(crate) mod element;
pub(crate) mod expression;
pub(crate) mod identifier;
pub(crate) mod keyword;
pub(crate) mod stream;
pub(crate) mod tag;
pub(crate) mod text;
pub(crate) mod r#type;
pub(crate) mod variable;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_document(&mut self, span: Span) -> Document<'arena> {
        let mut elements = self.new_vec::<Element<'arena>>();
        while !self.stream.has_reached_eof() {
            elements.push(self.parse_element());
        }

        let trivia = self.stream.take_trivia();
        let errors = self.take_errors();

        Document { span, trivia: Sequence::new(trivia), elements: Sequence::new(elements), errors: errors.leak() }
    }
}
