use crate::T;
use crate::ast::ast::FunctionLikeParameter;
use crate::ast::ast::FunctionLikeParameterDefaultValue;
use crate::ast::ast::FunctionLikeParameterList;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_optional_function_like_parameter_list(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<FunctionLikeParameterList<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["("]) => Some(self.parse_function_like_parameter_list(stream)?),
            _ => None,
        })
    }

    pub(crate) fn parse_function_like_parameter_list(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<FunctionLikeParameterList<'arena>, ParseError> {
        let result =
            self.parse_comma_separated_sequence(stream, T!["("], T![")"], Self::parse_function_like_parameter)?;

        Ok(FunctionLikeParameterList {
            left_parenthesis: result.open,
            parameters: result.sequence,
            right_parenthesis: result.close,
        })
    }

    pub(crate) fn parse_function_like_parameter(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<FunctionLikeParameter<'arena>, ParseError> {
        Ok(FunctionLikeParameter {
            attribute_lists: self.parse_attribute_list_sequence(stream)?,
            modifiers: self.parse_modifier_sequence(stream)?,
            hint: self.parse_optional_type_hint(stream)?,
            ampersand: if stream.is_at(T!["&"])? { Some(stream.eat(T!["&"])?.span) } else { None },
            ellipsis: if stream.is_at(T!["..."])? { Some(stream.eat(T!["..."])?.span) } else { None },
            variable: self.parse_direct_variable(stream)?,
            default_value: self.parse_optional_function_like_parameter_default_value(stream)?,
            hooks: self.parse_optional_property_hook_list(stream)?,
        })
    }

    fn parse_optional_function_like_parameter_default_value(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<FunctionLikeParameterDefaultValue<'arena>>, ParseError> {
        Ok(if stream.is_at(T!["="])? {
            let equals = stream.eat(T!["="])?.span;
            Some(FunctionLikeParameterDefaultValue { equals, value: self.parse_expression(stream)? })
        } else {
            None
        })
    }
}
