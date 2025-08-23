use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::block::parse_block;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::type_hint::parse_type_hint;
use crate::parser::internal::utils;
use crate::parser::internal::variable::parse_direct_variable;

pub fn parse_try<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Try<'arena>, ParseError> {
    Ok(Try {
        r#try: utils::expect_keyword(stream, T!["try"])?,
        block: parse_block(stream)?,
        catch_clauses: {
            let mut catch_clauses = stream.new_vec();
            while let Some(clause) = parse_optional_try_catch_clause(stream)? {
                catch_clauses.push(clause);
            }

            Sequence::new(catch_clauses)
        },
        finally_clause: parse_optional_try_finally_clause(stream)?,
    })
}

pub fn parse_optional_try_catch_clause<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<TryCatchClause<'arena>>, ParseError> {
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

pub fn parse_optional_try_finally_clause<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<TryFinallyClause<'arena>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["finally"]) => {
            Some(TryFinallyClause { finally: utils::expect_any_keyword(stream)?, block: parse_block(stream)? })
        }
        _ => None,
    })
}
