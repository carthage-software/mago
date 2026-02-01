use crate::T;
use crate::ast::ast::FunctionLikeParameter;
use crate::ast::ast::FunctionLikeParameterDefaultValue;
use crate::ast::ast::FunctionLikeParameterList;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_optional_function_like_parameter_list(
        &mut self,
    ) -> Result<Option<FunctionLikeParameterList<'arena>>, ParseError> {
        Ok(match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["("]) => Some(self.parse_function_like_parameter_list()?),
            _ => None,
        })
    }

    pub(crate) fn parse_function_like_parameter_list(
        &mut self,
    ) -> Result<FunctionLikeParameterList<'arena>, ParseError> {
        let result = self.parse_comma_separated_sequence(T!["("], T![")"], Self::parse_function_like_parameter)?;

        Ok(FunctionLikeParameterList {
            left_parenthesis: result.open,
            parameters: result.sequence,
            right_parenthesis: result.close,
        })
    }

    pub(crate) fn parse_function_like_parameter(&mut self) -> Result<FunctionLikeParameter<'arena>, ParseError> {
        Ok(FunctionLikeParameter {
            attribute_lists: self.parse_attribute_list_sequence()?,
            modifiers: self.parse_modifier_sequence()?,
            hint: self.parse_optional_type_hint()?,
            ampersand: if self.stream.is_at(T!["&"])? { Some(self.stream.eat(T!["&"])?.span) } else { None },
            ellipsis: if self.stream.is_at(T!["..."])? { Some(self.stream.eat(T!["..."])?.span) } else { None },
            variable: self.parse_direct_variable()?,
            default_value: self.parse_optional_function_like_parameter_default_value()?,
            hooks: self.parse_optional_property_hook_list()?,
        })
    }

    fn parse_optional_function_like_parameter_default_value(
        &mut self,
    ) -> Result<Option<FunctionLikeParameterDefaultValue<'arena>>, ParseError> {
        Ok(if self.stream.is_at(T!["="])? {
            let equals = self.stream.eat(T!["="])?.span;
            Some(FunctionLikeParameterDefaultValue { equals, value: self.parse_expression()? })
        } else {
            None
        })
    }
}
