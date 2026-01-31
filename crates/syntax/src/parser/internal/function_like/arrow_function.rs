use crate::T;
use crate::ast::ast::ArrowFunction;
use crate::ast::ast::AttributeList;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_arrow_function_with_attributes(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<ArrowFunction<'arena>, ParseError> {
        Ok(ArrowFunction {
            attribute_lists: attributes,
            r#static: self.maybe_expect_keyword(T!["static"])?,
            r#fn: self.expect_keyword(T!["fn"])?,
            ampersand: if self.stream.is_at(T!["&"])? { Some(self.stream.eat(T!["&"])?.span) } else { None },
            parameter_list: self.parse_function_like_parameter_list()?,
            return_type_hint: self.parse_optional_function_like_return_type_hint()?,
            arrow: self.stream.eat(T!["=>"])?.span,
            expression: self.arena.alloc(self.parse_expression()?),
        })
    }
}
