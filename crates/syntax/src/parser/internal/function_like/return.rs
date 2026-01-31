use crate::T;
use crate::ast::ast::FunctionLikeReturnTypeHint;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_optional_function_like_return_type_hint(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<FunctionLikeReturnTypeHint<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![":"]) => Some(self.parse_function_like_return_type_hint(stream)?),
            _ => None,
        })
    }

    pub(crate) fn parse_function_like_return_type_hint(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<FunctionLikeReturnTypeHint<'arena>, ParseError> {
        Ok(FunctionLikeReturnTypeHint { colon: stream.eat(T![":"])?.span, hint: self.parse_type_hint(stream)? })
    }
}
