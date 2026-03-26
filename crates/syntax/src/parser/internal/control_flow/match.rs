use crate::T;
use crate::ast::ast::Match;
use crate::ast::ast::MatchArm;
use crate::ast::ast::MatchDefaultArm;
use crate::ast::ast::MatchExpressionArm;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_match(&mut self) -> Result<Match<'arena>, ParseError> {
        let r#match = self.expect_keyword(T!["match"])?;
        let left_parenthesis = self.stream.eat_span(T!["("])?;
        let expression = self.arena.alloc(self.parse_expression()?);
        let right_parenthesis = self.stream.eat_span(T![")"])?;
        let arms_result = self.parse_comma_separated_sequence(T!["{"], T!["}"], |p| p.parse_match_arm())?;

        Ok(Match {
            r#match,
            left_parenthesis,
            expression,
            right_parenthesis,
            left_brace: arms_result.open,
            arms: arms_result.sequence,
            right_brace: arms_result.close,
        })
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            T!["default"] => MatchArm::Default(self.parse_match_default_arm()?),
            _ => MatchArm::Expression(self.parse_match_expression_arm()?),
        })
    }

    fn parse_match_expression_arm(&mut self) -> Result<MatchExpressionArm<'arena>, ParseError> {
        Ok(MatchExpressionArm {
            conditions: {
                let mut conditions = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(self.stream.peek_kind(0)?, Some(T!["=>"])) {
                        break;
                    }

                    conditions.push(self.parse_expression()?);

                    match self.stream.peek_kind(0)? {
                        Some(T![","]) => {
                            commas.push(self.stream.consume()?);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(conditions, commas)
            },
            arrow: self.stream.eat_span(T!["=>"])?,
            expression: self.arena.alloc(self.parse_expression()?),
        })
    }

    fn parse_match_default_arm(&mut self) -> Result<MatchDefaultArm<'arena>, ParseError> {
        Ok(MatchDefaultArm {
            default: self.expect_keyword(T!["default"])?,
            arrow: self.stream.eat_span(T!["=>"])?,
            expression: self.arena.alloc(self.parse_expression()?),
        })
    }
}
