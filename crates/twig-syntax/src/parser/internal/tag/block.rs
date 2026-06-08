use crate::ast::Block;
use crate::ast::BlockBody;
use crate::ast::BlockLong;
use crate::ast::BlockShort;
use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::internal::BlockTerminator;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;
use mago_allocator::prelude::*;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_block(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError<'arena>> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let name = self.expect_flexible_identifier(b"expected block name")?;

        if !self.stream.is_block_end()? {
            let expression = self.parse_expression()?;
            let close_tag = self.stream.expect_block_end()?;
            return Ok(Statement::Block(Block {
                open_tag,
                keyword,
                name,
                body: BlockBody::Short(BlockShort { expression, close_tag }),
            }));
        }

        let close_tag = self.stream.expect_block_end()?;
        let body = self.parse_statements(&BlockTerminator { names: &[b"endblock"] })?;
        let end_open_tok = self.stream.expect_block_start()?;
        let end_open_tag = self.stream.span_of(&end_open_tok);
        let end_kw_tok = self.stream.expect_name(b"expected `endblock`")?;
        if end_kw_tok.value != b"endblock" {
            return Err(ParseError::MismatchedEndTag {
                expected: b"endblock",
                got: end_kw_tok.value,
                span: self.stream.span_of(&end_kw_tok),
            });
        }
        let end_keyword = self.keyword_from(&end_kw_tok);

        let end_name = if let Some(closing_tok) = self.stream.try_consume(TwigTokenKind::Name)? {
            if closing_tok.value != name.value {
                return Err(ParseError::MismatchedEndTag {
                    expected: name.value,
                    got: closing_tok.value,
                    span: self.stream.span_of(&closing_tok),
                });
            }
            Some(self.identifier_from(&closing_tok))
        } else {
            None
        };
        let end_close_tag = self.stream.expect_block_end()?;

        Ok(Statement::Block(Block {
            open_tag,
            keyword,
            name,
            body: BlockBody::Long(BlockLong { close_tag, body, end_open_tag, end_keyword, end_name, end_close_tag }),
        }))
    }
}
