use crate::T;
use crate::ast::ast::AttributeList;
use crate::ast::ast::Method;
use crate::ast::ast::MethodAbstractBody;
use crate::ast::ast::MethodBody;
use crate::ast::ast::Modifier;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_method_with_attributes_and_modifiers(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
        modifiers: Sequence<'arena, Modifier<'arena>>,
    ) -> Result<Method<'arena>, ParseError> {
        Ok(Method {
            attribute_lists: attributes,
            modifiers,
            function: self.expect_keyword(stream, T!["function"])?,
            ampersand: if stream.is_at(T!["&"])? { Some(stream.eat(T!["&"])?.span) } else { None },
            name: self.parse_local_identifier(stream)?,
            parameter_list: self.parse_function_like_parameter_list(stream)?,
            return_type_hint: self.parse_optional_function_like_return_type_hint(stream)?,
            body: self.parse_method_body(stream)?,
        })
    }

    fn parse_method_body(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<MethodBody<'arena>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![";"]) => MethodBody::Abstract(MethodAbstractBody { semicolon: stream.consume()?.span }),
            _ => MethodBody::Concrete(self.parse_block(stream)?),
        })
    }
}
