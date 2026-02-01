use crate::T;
use crate::ast::ast::Try;
use crate::ast::ast::TryCatchClause;
use crate::ast::ast::TryFinallyClause;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_try(&mut self) -> Result<Try<'arena>, ParseError> {
        Ok(Try {
            r#try: self.expect_keyword(T!["try"])?,
            block: self.parse_block()?,
            catch_clauses: {
                let mut catch_clauses = self.new_vec();
                while let Some(clause) = self.parse_optional_try_catch_clause()? {
                    catch_clauses.push(clause);
                }

                Sequence::new(catch_clauses)
            },
            finally_clause: self.parse_optional_try_finally_clause()?,
        })
    }

    pub(crate) fn parse_optional_try_catch_clause(&mut self) -> Result<Option<TryCatchClause<'arena>>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T!["catch"]) => {
                let catch = self.expect_any_keyword()?;
                let left_parenthesis = self.stream.eat_span(T!["("])?;
                let hint = self.parse_type_hint()?;
                let var = match self.stream.peek_kind(0)? {
                    Some(T!["$variable"]) => Some(self.parse_direct_variable()?),
                    _ => None,
                };
                let right_parenthesis = self.stream.eat_span(T![")"])?;
                let blk = self.parse_block()?;

                Some(TryCatchClause { catch, left_parenthesis, hint, variable: var, right_parenthesis, block: blk })
            }
            _ => None,
        })
    }

    pub(crate) fn parse_optional_try_finally_clause(&mut self) -> Result<Option<TryFinallyClause<'arena>>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T!["finally"]) => {
                Some(TryFinallyClause { finally: self.expect_any_keyword()?, block: self.parse_block()? })
            }
            _ => None,
        })
    }
}
