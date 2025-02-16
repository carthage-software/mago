use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::statement::parse_statement;
use crate::internal::terminator::parse_optional_terminator;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_switch<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Switch<'i>, ParseError> {
    let switch = utils::expect_keyword(stream, T!["switch"])?;
    let left_parenthesis = utils::expect_span(stream, T!["("])?;
    let expression = parse_expression(stream)?;
    let right_parenthesis = utils::expect_span(stream, T![")"])?;
    let body = parse_switch_body(stream)?;

    Ok(Switch { switch, left_parenthesis, expression: stream.boxed(expression), right_parenthesis, body })
}

#[inline]
pub fn parse_switch_body<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<SwitchBody<'i>, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match token.kind {
        T![":"] => SwitchBody::ColonDelimited(parse_switch_colon_delimited_body(stream)?),
        T!["{"] => SwitchBody::BraceDelimited(parse_switch_brace_delimited_body(stream)?),
        _ => {
            return Err(utils::unexpected(stream, Some(token), T![":", "{"]));
        }
    })
}

#[inline]
pub fn parse_switch_brace_delimited_body<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<SwitchBraceDelimitedBody<'i>, ParseError> {
    let left_brace = utils::expect_span(stream, T!["{"])?;
    let optional_terminator = parse_optional_terminator(stream)?;
    let cases = parse_switch_cases(stream)?;
    let right_brace = utils::expect_span(stream, T!["}"])?;

    Ok(SwitchBraceDelimitedBody { left_brace, optional_terminator, cases, right_brace })
}

#[inline]
pub fn parse_switch_colon_delimited_body<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<SwitchColonDelimitedBody<'i>, ParseError> {
    Ok(SwitchColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        optional_terminator: parse_optional_terminator(stream)?,
        cases: parse_switch_cases(stream)?,
        end_switch: utils::expect_keyword(stream, T!["endswitch"])?,
        terminator: parse_terminator(stream)?,
    })
}

#[inline]
pub fn parse_switch_cases<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Sequence<'i, SwitchCase<'i>>, ParseError> {
    let mut cases = stream.vec();
    loop {
        if matches!(utils::peek(stream)?.kind, T!["endswitch" | "}"]) {
            break;
        }

        cases.push(parse_switch_case(stream)?);
    }

    Ok(Sequence::new(cases))
}

#[inline]
pub fn parse_switch_case<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<SwitchCase<'i>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T!["default"] => SwitchCase::Default(parse_switch_default_case(stream)?),
        _ => SwitchCase::Expression(parse_switch_expression_case(stream)?),
    })
}

#[inline]
pub fn parse_switch_expression_case<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<SwitchExpressionCase<'i>, ParseError> {
    let case = utils::expect_keyword(stream, T!["case"])?;
    let expression = parse_expression(stream)?;
    let separator = parse_switch_case_separator(stream)?;
    let statements = parse_switch_statements(stream)?;

    Ok(SwitchExpressionCase { case, expression: stream.boxed(expression), separator, statements })
}

#[inline]
pub fn parse_switch_default_case<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<SwitchDefaultCase<'i>, ParseError> {
    Ok(SwitchDefaultCase {
        default: utils::expect_keyword(stream, T!["default"])?,
        separator: parse_switch_case_separator(stream)?,
        statements: parse_switch_statements(stream)?,
    })
}

#[inline]
pub fn parse_switch_statements<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Sequence<'i, Statement<'i>>, ParseError> {
    let mut statements = stream.vec();
    loop {
        if matches!(utils::peek(stream)?.kind, T!["case" | "default" | "endswitch" | "}"]) {
            break;
        }

        statements.push(parse_statement(stream)?);
    }

    Ok(Sequence::new(statements))
}

#[inline]
pub fn parse_switch_case_separator(stream: &mut TokenStream<'_, '_>) -> Result<SwitchCaseSeparator, ParseError> {
    let token = utils::expect_one_of(stream, T![":", ";"])?;

    Ok(match token.kind {
        T![":"] => SwitchCaseSeparator::Colon(token.span),
        T![";"] => SwitchCaseSeparator::SemiColon(token.span),
        _ => unreachable!(),
    })
}
