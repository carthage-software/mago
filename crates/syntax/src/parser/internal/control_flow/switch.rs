use mago_allocator::prelude::*;
use mago_database::file::HasFileId;
use mago_span::Span;

use crate::T;
use crate::cst::cst::Statement;
use crate::cst::cst::Switch;
use crate::cst::cst::SwitchBody;
use crate::cst::cst::SwitchBraceDelimitedBody;
use crate::cst::cst::SwitchCase;
use crate::cst::cst::SwitchCaseSeparator;
use crate::cst::cst::SwitchColonDelimitedBody;
use crate::cst::cst::SwitchDefaultCase;
use crate::cst::cst::SwitchExpressionCase;
use crate::cst::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_switch(&mut self) -> Result<Switch<'arena>, ParseError> {
        let switch = self.expect_keyword(T!["switch"])?;
        let left_parenthesis = self.stream.eat_span(T!["("])?;
        let expression = self.parse_expression()?;
        let right_parenthesis = self.parse_condition_closing_span()?;

        Ok(Switch {
            switch,
            left_parenthesis,
            expression,
            right_parenthesis,
            body: self.parse_switch_body(right_parenthesis)?,
        })
    }

    fn parse_switch_body(&mut self, right_parenthesis: Span) -> Result<SwitchBody<'arena>, ParseError> {
        let token = match self.stream.lookahead(0)? {
            Some(token) => token,
            None if right_parenthesis.length() == 0 => return Ok(missing_switch_body(right_parenthesis)),
            None => return Ok(missing_switch_body(self.recover_at_eof(self.stream.unexpected(None, &[]))?)),
        };

        Ok(match token.kind {
            T![":"] => SwitchBody::ColonDelimited(self.parse_switch_colon_delimited_body()?),
            T!["{"] => SwitchBody::BraceDelimited(self.parse_switch_brace_delimited_body()?),
            _ if right_parenthesis.length() == 0 => missing_switch_body(right_parenthesis),
            _ if self.recovery_end.is_some() => {
                self.errors.push(self.stream.unexpected(Some(token), T![":", "{"]));
                missing_switch_body(self.empty_span())
            }
            _ => {
                return Err(self.stream.unexpected(Some(token), T![":", "{"]));
            }
        })
    }

    fn parse_switch_brace_delimited_body(&mut self) -> Result<SwitchBraceDelimitedBody<'arena>, ParseError> {
        let left_brace = self.stream.eat_span(T!["{"])?;
        let optional_terminator = self.parse_optional_terminator()?;
        let cases = self.parse_switch_cases()?;
        let right_brace = self.stream.eat_span(T!["}"])?;

        Ok(SwitchBraceDelimitedBody { left_brace, optional_terminator, cases, right_brace })
    }

    fn parse_switch_colon_delimited_body(&mut self) -> Result<SwitchColonDelimitedBody<'arena>, ParseError> {
        Ok(SwitchColonDelimitedBody {
            colon: self.stream.eat_span(T![":"])?,
            optional_terminator: self.parse_optional_terminator()?,
            cases: self.parse_switch_cases()?,
            end_switch: self.expect_keyword(T!["endswitch"])?,
            terminator: self.parse_terminator()?,
        })
    }

    fn parse_switch_cases(&mut self) -> Result<Sequence<'arena, SwitchCase<'arena>>, ParseError> {
        let mut cases = self.new_vec();
        loop {
            if matches!(self.stream.peek_kind(0)?, Some(T!["endswitch" | "}"])) {
                break;
            }

            cases.push(self.parse_switch_case()?);
        }

        Ok(Sequence::new(cases))
    }

    fn parse_switch_case(&mut self) -> Result<SwitchCase<'arena>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T!["default"]) => SwitchCase::Default(self.parse_switch_default_case()?),
            _ => SwitchCase::Expression(self.parse_switch_expression_case()?),
        })
    }

    fn parse_switch_expression_case(&mut self) -> Result<SwitchExpressionCase<'arena>, ParseError> {
        Ok(SwitchExpressionCase {
            case: self.expect_keyword(T!["case"])?,
            expression: self.parse_expression()?,
            separator: self.parse_switch_case_separator()?,
            statements: self.parse_switch_statements()?,
        })
    }

    fn parse_switch_default_case(&mut self) -> Result<SwitchDefaultCase<'arena>, ParseError> {
        Ok(SwitchDefaultCase {
            default: self.expect_keyword(T!["default"])?,
            separator: self.parse_switch_case_separator()?,
            statements: self.parse_switch_statements()?,
        })
    }

    fn parse_switch_statements(&mut self) -> Result<Sequence<'arena, Statement<'arena>>, ParseError> {
        let mut statements = self.new_vec();
        loop {
            if matches!(self.stream.peek_kind(0)?, None | Some(T!["case" | "default" | "endswitch" | "}"])) {
                break;
            }
            let position_before = self.stream.current_position();
            statements.push(self.parse_statement()?);
            if self.stream.current_position() == position_before {
                if let Ok(Some(token)) = self.stream.lookahead(0) {
                    self.errors.push(self.stream.unexpected(Some(token), &[]));
                    let _ = self.stream.consume();
                } else {
                    break;
                }
            }
        }

        Ok(Sequence::new(statements))
    }

    fn parse_switch_case_separator(&mut self) -> Result<SwitchCaseSeparator, ParseError> {
        let token = self.stream.consume()?;
        let span = token.span_for(self.stream.file_id());

        Ok(match token.kind {
            T![":"] => SwitchCaseSeparator::Colon(span),
            T![";"] => SwitchCaseSeparator::SemiColon(span),
            _ => return Err(self.stream.unexpected(Some(token), T![":", ";"])),
        })
    }
}

fn missing_switch_body<'arena>(span: Span) -> SwitchBody<'arena> {
    SwitchBody::BraceDelimited(SwitchBraceDelimitedBody {
        left_brace: span,
        optional_terminator: None,
        cases: Sequence::empty(),
        right_brace: span,
    })
}
