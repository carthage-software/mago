use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_match<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Match<'arena>, ParseError> {
    Ok(Match {
        r#match: utils::expect_keyword(stream, T!["match"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        expression: {
            let expression = parse_expression(stream)?;

            stream.alloc(expression)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        left_brace: utils::expect_span(stream, T!["{"])?,
        arms: {
            let mut arms = stream.new_vec();
            let mut commas = stream.new_vec();
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
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

pub fn parse_match_arm<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<MatchArm<'arena>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T!["default"] => MatchArm::Default(parse_match_default_arm(stream)?),
        _ => MatchArm::Expression(parse_match_expression_arm(stream)?),
    })
}

pub fn parse_match_expression_arm<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<MatchExpressionArm<'arena>, ParseError> {
    Ok(MatchExpressionArm {
        conditions: {
            let mut conditions = stream.new_vec();
            let mut commas = stream.new_vec();
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
        },
        arrow: utils::expect_span(stream, T!["=>"])?,
        expression: {
            let expression = parse_expression(stream)?;

            stream.alloc(expression)
        },
    })
}

pub fn parse_match_default_arm<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<MatchDefaultArm<'arena>, ParseError> {
    Ok(MatchDefaultArm {
        default: utils::expect_keyword(stream, T!["default"])?,
        arrow: utils::expect_span(stream, T!["=>"])?,
        expression: {
            let expression = parse_expression(stream)?;

            stream.alloc(expression)
        },
    })
}
