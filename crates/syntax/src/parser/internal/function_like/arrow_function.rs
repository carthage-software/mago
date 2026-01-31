use crate::T;
use crate::ast::ast::ArrowFunction;
use crate::ast::ast::AttributeList;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_arrow_function_with_attributes(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<ArrowFunction<'arena>, ParseError> {
        Ok(ArrowFunction {
            attribute_lists: attributes,
            r#static: self.maybe_expect_keyword(stream, T!["static"])?,
            r#fn: self.expect_keyword(stream, T!["fn"])?,
            ampersand: if stream.is_at(T!["&"])? { Some(stream.eat(T!["&"])?.span) } else { None },
            parameter_list: self.parse_function_like_parameter_list(stream)?,
            return_type_hint: self.parse_optional_function_like_return_type_hint(stream)?,
            arrow: stream.eat(T!["=>"])?.span,
            expression: self.arena.alloc(self.parse_expression(stream)?),
        })
    }
}
