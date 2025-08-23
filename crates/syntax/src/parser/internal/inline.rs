use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_inline<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Inline<'arena>, ParseError> {
    let token = utils::expect_one_of(stream, T![InlineText, InlineShebang])?;

    Ok(Inline {
        kind: if token.kind == T![InlineShebang] { InlineKind::Shebang } else { InlineKind::Text },
        span: token.span,
        value: token.value,
    })
}
