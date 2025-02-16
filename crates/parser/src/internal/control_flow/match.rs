use mago_ast::ast::*;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_match<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Match<'i>, ParseError> {
    let r#match = utils::expect_keyword(stream, T!["match"])?;
    let left_parenthesis = utils::expect_span(stream, T!["("])?;
    let expression = parse_expression(stream)?;
    let right_parenthesis = utils::expect_span(stream, T![")"])?;
    let left_brace = utils::expect_span(stream, T!["{"])?;
    let arms = {
        let mut arms = stream.vec();
        let mut commas = stream.vec();
        loop {
            if matches!(utils::peek(stream)?.kind, T!["}"]) {
                break;
            }

            arms.push(parse_match_arm(stream)?);

            match utils::peek(stream)?.kind {
                T![","] => {
                    commas.push(utils::expect_any(stream)?);
                }
                _ => {
                    break;
                }
            }
        }

        TokenSeparatedSequence::new(arms, commas)
    };
    let right_brace = utils::expect_span(stream, T!["}"])?;

    Ok(Match {
        r#match,
        left_parenthesis,
        expression: stream.boxed(expression),
        right_parenthesis,
        left_brace,
        arms,
        right_brace,
    })
}

#[inline]
pub fn parse_match_arm<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<MatchArm<'i>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T!["default"] => MatchArm::Default(parse_match_default_arm(stream)?),
        _ => MatchArm::Expression(parse_match_expression_arm(stream)?),
    })
}

#[inline]
pub fn parse_match_expression_arm<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<MatchExpressionArm<'i>, ParseError> {
    let conditions = {
        let mut conditions = stream.vec();
        let mut commas = stream.vec();
        loop {
            if matches!(utils::peek(stream)?.kind, T!["=>"]) {
                break;
            }

            conditions.push(parse_expression(stream)?);

            match utils::peek(stream)?.kind {
                T![","] => {
                    commas.push(utils::expect_any(stream)?);
                }
                _ => {
                    break;
                }
            }
        }

        TokenSeparatedSequence::new(conditions, commas)
    };
    let arrow = utils::expect_span(stream, T!["=>"])?;
    let expression = parse_expression(stream)?;

    Ok(MatchExpressionArm { conditions, arrow, expression: stream.boxed(expression) })
}

#[inline]
pub fn parse_match_default_arm<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<MatchDefaultArm<'i>, ParseError> {
    let default = utils::expect_keyword(stream, T!["default"])?;
    let arrow = utils::expect_span(stream, T!["=>"])?;
    let expression = parse_expression(stream)?;

    Ok(MatchDefaultArm { default, arrow, expression: stream.boxed(expression) })
}
