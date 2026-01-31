use crate::T;
use crate::ast::ast::Match;
use crate::ast::ast::MatchArm;
use crate::ast::ast::MatchDefaultArm;
use crate::ast::ast::MatchExpressionArm;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_match(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Match<'arena>, ParseError> {
        let r#match = self.expect_keyword(stream, T!["match"])?;
        let left_parenthesis = stream.eat(T!["("])?.span;
        let expression = self.arena.alloc(self.parse_expression(stream)?);
        let right_parenthesis = stream.eat(T![")"])?.span;
        let arms_result = self.parse_comma_separated_sequence(stream, T!["{"], T!["}"], |p, s| p.parse_match_arm(s))?;

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

    fn parse_match_arm(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<MatchArm<'arena>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            T!["default"] => MatchArm::Default(self.parse_match_default_arm(stream)?),
            _ => MatchArm::Expression(self.parse_match_expression_arm(stream)?),
        })
    }

    fn parse_match_expression_arm(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<MatchExpressionArm<'arena>, ParseError> {
        Ok(MatchExpressionArm {
            conditions: {
                let mut conditions = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["=>"])) {
                        break;
                    }

                    conditions.push(self.parse_expression(stream)?);

                    match stream.lookahead(0)?.map(|t| t.kind) {
                        Some(T![","]) => {
                            commas.push(stream.consume()?);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(conditions, commas)
            },
            arrow: stream.eat(T!["=>"])?.span,
            expression: self.arena.alloc(self.parse_expression(stream)?),
        })
    }

    fn parse_match_default_arm(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<MatchDefaultArm<'arena>, ParseError> {
        Ok(MatchDefaultArm {
            default: self.expect_keyword(stream, T!["default"])?,
            arrow: stream.eat(T!["=>"])?.span,
            expression: self.arena.alloc(self.parse_expression(stream)?),
        })
    }
}
