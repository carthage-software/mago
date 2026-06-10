use mago_allocator::Arena;
use mago_span::Span;

use crate::cst::r#type::SliceType;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_slice_type(
        &mut self,
        inner: &'arena Type<'arena>,
        left_bracket: Span,
    ) -> Result<Type<'arena>, ParseError> {
        let right_bracket = self.stream.consume_span()?;

        Ok(Type::Slice(SliceType { inner, left_bracket, right_bracket }))
    }
}
