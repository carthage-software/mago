use crate::T;
use crate::ast::ast::Statement;
use crate::ast::ast::Switch;
use crate::ast::ast::SwitchBody;
use crate::ast::ast::SwitchBraceDelimitedBody;
use crate::ast::ast::SwitchCase;
use crate::ast::ast::SwitchCaseSeparator;
use crate::ast::ast::SwitchColonDelimitedBody;
use crate::ast::ast::SwitchDefaultCase;
use crate::ast::ast::SwitchExpressionCase;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_switch(&mut self) -> Result<Switch<'arena>, ParseError> {
        Ok(Switch {
            switch: self.expect_keyword(T!["switch"])?,
            left_parenthesis: self.stream.eat(T!["("])?.span,
            expression: self.arena.alloc(self.parse_expression()?),
            right_parenthesis: self.stream.eat(T![")"])?.span,
            body: self.parse_switch_body()?,
        })
    }

    fn parse_switch_body(&mut self) -> Result<SwitchBody<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        Ok(match token.kind {
            T![":"] => SwitchBody::ColonDelimited(self.parse_switch_colon_delimited_body()?),
            T!["{"] => SwitchBody::BraceDelimited(self.parse_switch_brace_delimited_body()?),
            _ => {
                return Err(self.stream.unexpected(Some(token), T![":", "{"]));
            }
        })
    }

    fn parse_switch_brace_delimited_body(&mut self) -> Result<SwitchBraceDelimitedBody<'arena>, ParseError> {
        let left_brace = self.stream.eat(T!["{"])?.span;
        let optional_terminator = self.parse_optional_terminator()?;
        let cases = self.parse_switch_cases()?;
        let right_brace = self.stream.eat(T!["}"])?.span;

        Ok(SwitchBraceDelimitedBody { left_brace, optional_terminator, cases, right_brace })
    }

    fn parse_switch_colon_delimited_body(&mut self) -> Result<SwitchColonDelimitedBody<'arena>, ParseError> {
        Ok(SwitchColonDelimitedBody {
            colon: self.stream.eat(T![":"])?.span,
            optional_terminator: self.parse_optional_terminator()?,
            cases: self.parse_switch_cases()?,
            end_switch: self.expect_keyword(T!["endswitch"])?,
            terminator: self.parse_terminator()?,
        })
    }

    fn parse_switch_cases(&mut self) -> Result<Sequence<'arena, SwitchCase<'arena>>, ParseError> {
        let mut cases = self.new_vec();
        loop {
            if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["endswitch" | "}"])) {
                break;
            }

            cases.push(self.parse_switch_case()?);
        }

        Ok(Sequence::new(cases))
    }

    fn parse_switch_case(&mut self) -> Result<SwitchCase<'arena>, ParseError> {
        Ok(match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["default"]) => SwitchCase::Default(self.parse_switch_default_case()?),
            _ => SwitchCase::Expression(self.parse_switch_expression_case()?),
        })
    }

    fn parse_switch_expression_case(&mut self) -> Result<SwitchExpressionCase<'arena>, ParseError> {
        Ok(SwitchExpressionCase {
            case: self.expect_keyword(T!["case"])?,
            expression: self.arena.alloc(self.parse_expression()?),
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
            if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["case" | "default" | "endswitch" | "}"])) {
                break;
            }

            statements.push(self.parse_statement()?);
        }

        Ok(Sequence::new(statements))
    }

    fn parse_switch_case_separator(&mut self) -> Result<SwitchCaseSeparator, ParseError> {
        let token = self.stream.consume()?;

        Ok(match token.kind {
            T![":"] => SwitchCaseSeparator::Colon(token.span),
            T![";"] => SwitchCaseSeparator::SemiColon(token.span),
            _ => return Err(self.stream.unexpected(Some(token), T![":", ";"])),
        })
    }
}
