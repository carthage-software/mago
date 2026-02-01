use crate::T;
use crate::ast::ast::FunctionLikeReturnTypeHint;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_optional_function_like_return_type_hint(
        &mut self,
    ) -> Result<Option<FunctionLikeReturnTypeHint<'arena>>, ParseError> {
        Ok(match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![":"]) => Some(self.parse_function_like_return_type_hint()?),
            _ => None,
        })
    }

    pub(crate) fn parse_function_like_return_type_hint(
        &mut self,
    ) -> Result<FunctionLikeReturnTypeHint<'arena>, ParseError> {
        Ok(FunctionLikeReturnTypeHint { colon: self.stream.eat(T![":"])?.span, hint: self.parse_type_hint()? })
    }
}
