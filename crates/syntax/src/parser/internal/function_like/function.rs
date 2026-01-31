use crate::T;
use crate::ast::ast::AttributeList;
use crate::ast::ast::Function;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_function_with_attributes(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Function<'arena>, ParseError> {
        Ok(Function {
            attribute_lists: attributes,
            function: self.expect_keyword(stream, T!["function"])?,
            ampersand: if stream.is_at(T!["&"])? { Some(stream.eat(T!["&"])?.span) } else { None },
            name: self.parse_local_identifier(stream)?,
            parameter_list: self.parse_function_like_parameter_list(stream)?,
            return_type_hint: self.parse_optional_function_like_return_type_hint(stream)?,
            body: self.parse_block(stream)?,
        })
    }
}
