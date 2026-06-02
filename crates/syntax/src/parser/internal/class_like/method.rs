use crate::T;
use crate::cst::cst::AttributeList;
use crate::cst::cst::Method;
use crate::cst::cst::MethodAbstractBody;
use crate::cst::cst::MethodBody;
use crate::cst::cst::Modifier;
use crate::cst::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_method_with_attributes_and_modifiers(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
        modifiers: Sequence<'arena, Modifier<'arena>>,
    ) -> Result<Method<'arena>, ParseError> {
        Ok(Method {
            attribute_lists: attributes,
            modifiers,
            function: self.expect_keyword(T!["function"])?,
            ampersand: if self.stream.is_at(T!["&"])? { Some(self.stream.eat_span(T!["&"])?) } else { None },
            name: self.parse_local_identifier()?,
            parameter_list: self.parse_function_like_parameter_list()?,
            return_type_hint: self.parse_optional_function_like_return_type_hint()?,
            body: self.parse_method_body()?,
        })
    }

    fn parse_method_body(&mut self) -> Result<MethodBody<'arena>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T![";"]) => MethodBody::Abstract(MethodAbstractBody { semicolon: self.stream.consume_span()? }),
            _ => MethodBody::Concrete(self.parse_block()?),
        })
    }
}
