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
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_switch(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Switch<'arena>, ParseError> {
        Ok(Switch {
            switch: self.expect_keyword(stream, T!["switch"])?,
            left_parenthesis: stream.eat(T!["("])?.span,
            expression: self.arena.alloc(self.parse_expression(stream)?),
            right_parenthesis: stream.eat(T![")"])?.span,
            body: self.parse_switch_body(stream)?,
        })
    }

    fn parse_switch_body(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<SwitchBody<'arena>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        Ok(match token.kind {
            T![":"] => SwitchBody::ColonDelimited(self.parse_switch_colon_delimited_body(stream)?),
            T!["{"] => SwitchBody::BraceDelimited(self.parse_switch_brace_delimited_body(stream)?),
            _ => {
                return Err(stream.unexpected(Some(token), T![":", "{"]));
            }
        })
    }

    fn parse_switch_brace_delimited_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<SwitchBraceDelimitedBody<'arena>, ParseError> {
        let left_brace = stream.eat(T!["{"])?.span;
        let optional_terminator = self.parse_optional_terminator(stream)?;
        let cases = self.parse_switch_cases(stream)?;
        let right_brace = stream.eat(T!["}"])?.span;

        Ok(SwitchBraceDelimitedBody { left_brace, optional_terminator, cases, right_brace })
    }

    fn parse_switch_colon_delimited_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<SwitchColonDelimitedBody<'arena>, ParseError> {
        Ok(SwitchColonDelimitedBody {
            colon: stream.eat(T![":"])?.span,
            optional_terminator: self.parse_optional_terminator(stream)?,
            cases: self.parse_switch_cases(stream)?,
            end_switch: self.expect_keyword(stream, T!["endswitch"])?,
            terminator: self.parse_terminator(stream)?,
        })
    }

    fn parse_switch_cases(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Sequence<'arena, SwitchCase<'arena>>, ParseError> {
        let mut cases = self.new_vec();
        loop {
            if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["endswitch" | "}"])) {
                break;
            }

            cases.push(self.parse_switch_case(stream)?);
        }

        Ok(Sequence::new(cases))
    }

    fn parse_switch_case(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<SwitchCase<'arena>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["default"]) => SwitchCase::Default(self.parse_switch_default_case(stream)?),
            _ => SwitchCase::Expression(self.parse_switch_expression_case(stream)?),
        })
    }

    fn parse_switch_expression_case(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<SwitchExpressionCase<'arena>, ParseError> {
        Ok(SwitchExpressionCase {
            case: self.expect_keyword(stream, T!["case"])?,
            expression: self.arena.alloc(self.parse_expression(stream)?),
            separator: self.parse_switch_case_separator(stream)?,
            statements: self.parse_switch_statements(stream)?,
        })
    }

    fn parse_switch_default_case(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<SwitchDefaultCase<'arena>, ParseError> {
        Ok(SwitchDefaultCase {
            default: self.expect_keyword(stream, T!["default"])?,
            separator: self.parse_switch_case_separator(stream)?,
            statements: self.parse_switch_statements(stream)?,
        })
    }

    fn parse_switch_statements(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Sequence<'arena, Statement<'arena>>, ParseError> {
        let mut statements = self.new_vec();
        loop {
            if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["case" | "default" | "endswitch" | "}"])) {
                break;
            }

            statements.push(self.parse_statement(stream)?);
        }

        Ok(Sequence::new(statements))
    }

    fn parse_switch_case_separator(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<SwitchCaseSeparator, ParseError> {
        let token = stream.consume()?;

        Ok(match token.kind {
            T![":"] => SwitchCaseSeparator::Colon(token.span),
            T![";"] => SwitchCaseSeparator::SemiColon(token.span),
            _ => return Err(stream.unexpected(Some(token), T![":", ";"])),
        })
    }
}
