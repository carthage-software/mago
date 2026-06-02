use mago_span::Span;
use mago_syntax_core::ast::Sequence;

use crate::cst::Document;
use crate::cst::element::Element;
use crate::parser::PHPDocParser;

pub(crate) mod alloc;
pub(crate) mod element;
pub(crate) mod expression;
pub(crate) mod identifier;
pub(crate) mod keyword;
pub(crate) mod stream;
pub(crate) mod tag;
pub(crate) mod text;
pub(crate) mod r#type;
pub(crate) mod variable;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_document(&mut self, span: Span) -> Document<'arena> {
        let mut elements = self.new_vec::<Element<'arena>>();
        while !self.stream.has_reached_eof() {
            elements.push(self.parse_element());
        }

        let trivia = self.stream.take_trivia();
        let inherit_docs = self.take_inherit_docs();
        let errors = self.take_errors();

        Document {
            span,
            trivia: Sequence::new(trivia),
            elements: Sequence::new(elements),
            inherit_docs: Sequence::new(inherit_docs),
            errors,
        }
    }
}
