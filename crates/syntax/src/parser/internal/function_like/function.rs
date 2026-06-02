use crate::T;
use crate::cst::cst::AttributeList;
use crate::cst::cst::Function;
use crate::cst::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_function_with_attributes(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Function<'arena>, ParseError> {
        Ok(Function {
            attribute_lists: attributes,
            function: self.expect_keyword(T!["function"])?,
            ampersand: if self.stream.is_at(T!["&"])? { Some(self.stream.eat_span(T!["&"])?) } else { None },
            name: self.parse_local_identifier()?,
            parameter_list: self.parse_function_like_parameter_list()?,
            return_type_hint: self.parse_optional_function_like_return_type_hint()?,
            body: self.parse_block()?,
        })
    }
}
