use crate::ast::IgnoreMissingClause;
use crate::ast::Include;
use crate::ast::Keyword;
use crate::ast::Statement;
use crate::ast::WithExpressionClause;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;

pub(super) type IncludeTail<'arena> =
    (Option<IgnoreMissingClause<'arena>>, Option<WithExpressionClause<'arena>>, Option<Keyword<'arena>>);

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_include(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let template = self.parse_expression()?;
        let (ignore_missing, with_clause, only_keyword) = self.parse_include_tail()?;
        let close_tag = self.stream.expect_block_end()?;
        Ok(Statement::Include(Include {
            open_tag,
            keyword,
            template,
            ignore_missing,
            with_clause,
            only_keyword,
            close_tag,
        }))
    }

    pub(super) fn parse_include_tail(&mut self) -> Result<IncludeTail<'arena>, ParseError> {
        let mut ignore_missing = None;
        if let Some(ignore_keyword) = self.try_consume_name_keyword("ignore")? {
            let missing_tok = self.stream.expect_name_value("missing")?;
            let missing_keyword = self.keyword_from(&missing_tok);
            ignore_missing = Some(IgnoreMissingClause { ignore_keyword, missing_keyword });
        }
        let mut with_clause = None;
        if let Some(with_keyword) = self.try_consume_name_keyword("with")? {
            let variables = self.parse_expression()?;
            with_clause = Some(WithExpressionClause { with_keyword, variables });
        }
        let only_keyword = self.try_consume_name_keyword("only")?;
        Ok((ignore_missing, with_clause, only_keyword))
    }
}
