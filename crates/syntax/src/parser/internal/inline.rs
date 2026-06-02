use crate::T;
use crate::cst::cst::Inline;
use crate::cst::cst::InlineKind;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_inline(&mut self) -> Result<Inline<'arena>, ParseError> {
        let token = self.expect_one_of_keyword(T![InlineText, InlineShebang])?;

        Ok(Inline {
            kind: if token.span.start.offset == 0 && token.value.starts_with(b"#!") && token.value.contains(&b'\n') {
                InlineKind::Shebang
            } else {
                InlineKind::Text
            },
            span: token.span,
            value: token.value,
        })
    }
}
