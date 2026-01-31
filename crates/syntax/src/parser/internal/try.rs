use crate::T;
use crate::ast::ast::Try;
use crate::ast::ast::TryCatchClause;
use crate::ast::ast::TryFinallyClause;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_try(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Try<'arena>, ParseError> {
        Ok(Try {
            r#try: self.expect_keyword(stream, T!["try"])?,
            block: self.parse_block(stream)?,
            catch_clauses: {
                let mut catch_clauses = self.new_vec();
                while let Some(clause) = self.parse_optional_try_catch_clause(stream)? {
                    catch_clauses.push(clause);
                }

                Sequence::new(catch_clauses)
            },
            finally_clause: self.parse_optional_try_finally_clause(stream)?,
        })
    }

    pub(crate) fn parse_optional_try_catch_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<TryCatchClause<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["catch"]) => {
                let catch = self.expect_any_keyword(stream)?;
                let left_parenthesis = stream.eat(T!["("])?.span;
                let hint = self.parse_type_hint(stream)?;
                let var = match stream.lookahead(0)?.map(|t| t.kind) {
                    Some(T!["$variable"]) => Some(self.parse_direct_variable(stream)?),
                    _ => None,
                };
                let right_parenthesis = stream.eat(T![")"])?.span;
                let blk = self.parse_block(stream)?;

                Some(TryCatchClause { catch, left_parenthesis, hint, variable: var, right_parenthesis, block: blk })
            }
            _ => None,
        })
    }

    pub(crate) fn parse_optional_try_finally_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<TryFinallyClause<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["finally"]) => {
                Some(TryFinallyClause { finally: self.expect_any_keyword(stream)?, block: self.parse_block(stream)? })
            }
            _ => None,
        })
    }
}
