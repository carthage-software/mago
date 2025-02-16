use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::block::parse_block;
use crate::internal::token_stream::TokenStream;
use crate::internal::type_hint::parse_type_hint;
use crate::internal::utils;
use crate::internal::variable::parse_direct_variable;

#[inline]
pub fn parse_try<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Try<'i>, ParseError> {
    Ok(Try {
        r#try: utils::expect_keyword(stream, T!["try"])?,
        block: parse_block(stream)?,
        catch_clauses: {
            let mut catch_clauses = stream.vec();
            while let Some(clause) = parse_optional_try_catch_clause(stream)? {
                catch_clauses.push(clause);
            }

            Sequence::new(catch_clauses)
        },
        finally_clause: parse_optional_try_finally_clause(stream)?,
    })
}

#[inline]
pub fn parse_optional_try_catch_clause<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Option<TryCatchClause<'i>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["catch"]) => {
            let catch = utils::expect_any_keyword(stream)?;
            let left_parenthesis = utils::expect_span(stream, T!["("])?;
            let hint = parse_type_hint(stream)?;
            let variable = match utils::peek(stream)?.kind {
                T!["$variable"] => Some(parse_direct_variable(stream)?),
                _ => None,
            };
            let right_parenthesis = utils::expect_span(stream, T![")"])?;
            let block = parse_block(stream)?;

            Some(TryCatchClause { catch, left_parenthesis, hint, variable, right_parenthesis, block })
        }
        _ => None,
    })
}

#[inline]
pub fn parse_optional_try_finally_clause<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Option<TryFinallyClause<'i>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["finally"]) => {
            Some(TryFinallyClause { finally: utils::expect_any_keyword(stream)?, block: parse_block(stream)? })
        }
        _ => None,
    })
}
