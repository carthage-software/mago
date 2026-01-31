use crate::T;
use crate::ast::ast::Inline;
use crate::ast::ast::InlineKind;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_inline(&self, stream: &mut TokenStream<'_, 'arena>) -> Result<Inline<'arena>, ParseError> {
        let token = self.expect_one_of_keyword(stream, T![InlineText, InlineShebang])?;

        Ok(Inline {
            kind: if token.span.start.offset == 0 && token.value.starts_with("#!") && token.value.contains('\n') {
                InlineKind::Shebang
            } else {
                InlineKind::Text
            },
            span: token.span,
            value: token.value,
        })
    }
}
