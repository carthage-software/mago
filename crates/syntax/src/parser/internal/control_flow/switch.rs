use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::statement::parse_statement;
use crate::parser::internal::terminator::parse_optional_terminator;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_switch<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Switch<'arena>, ParseError> {
    Ok(Switch {
        switch: utils::expect_keyword(stream, T!["switch"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        expression: {
            let expression = parse_expression(stream)?;

            stream.alloc(expression)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        body: parse_switch_body(stream)?,
    })
}

pub fn parse_switch_body<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<SwitchBody<'arena>, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match token.kind {
        T![":"] => SwitchBody::ColonDelimited(parse_switch_colon_delimited_body(stream)?),
        T!["{"] => SwitchBody::BraceDelimited(parse_switch_brace_delimited_body(stream)?),
        _ => {
            return Err(utils::unexpected(stream, Some(token), T![":", "{"]));
        }
    })
}

pub fn parse_switch_brace_delimited_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<SwitchBraceDelimitedBody<'arena>, ParseError> {
    let left_brace = utils::expect_span(stream, T!["{"])?;
    let optional_terminator = parse_optional_terminator(stream)?;
    let cases = parse_switch_cases(stream)?;
    let right_brace = utils::expect_span(stream, T!["}"])?;

    Ok(SwitchBraceDelimitedBody { left_brace, optional_terminator, cases, right_brace })
}

pub fn parse_switch_colon_delimited_body<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<SwitchColonDelimitedBody<'arena>, ParseError> {
    Ok(SwitchColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        optional_terminator: parse_optional_terminator(stream)?,
        cases: parse_switch_cases(stream)?,
        end_switch: utils::expect_keyword(stream, T!["endswitch"])?,
        terminator: parse_terminator(stream)?,
    })
}

pub fn parse_switch_cases<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Sequence<'arena, SwitchCase<'arena>>, ParseError> {
    let mut cases = stream.new_vec();
    loop {
        if matches!(utils::peek(stream)?.kind, T!["endswitch" | "}"]) {
            break;
        }

        cases.push(parse_switch_case(stream)?);
    }

    Ok(Sequence::new(cases))
}

pub fn parse_switch_case<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<SwitchCase<'arena>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T!["default"] => SwitchCase::Default(parse_switch_default_case(stream)?),
        _ => SwitchCase::Expression(parse_switch_expression_case(stream)?),
    })
}

pub fn parse_switch_expression_case<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<SwitchExpressionCase<'arena>, ParseError> {
    Ok(SwitchExpressionCase {
        case: utils::expect_keyword(stream, T!["case"])?,
        expression: {
            let expression = parse_expression(stream)?;

            stream.alloc(expression)
        },
        separator: parse_switch_case_separator(stream)?,
        statements: parse_switch_statements(stream)?,
    })
}

pub fn parse_switch_default_case<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<SwitchDefaultCase<'arena>, ParseError> {
    Ok(SwitchDefaultCase {
        default: utils::expect_keyword(stream, T!["default"])?,
        separator: parse_switch_case_separator(stream)?,
        statements: parse_switch_statements(stream)?,
    })
}

pub fn parse_switch_statements<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Sequence<'arena, Statement<'arena>>, ParseError> {
    let mut statements = stream.new_vec();
    loop {
        if matches!(utils::peek(stream)?.kind, T!["case" | "default" | "endswitch" | "}"]) {
            break;
        }

        statements.push(parse_statement(stream)?);
    }

    Ok(Sequence::new(statements))
}

pub fn parse_switch_case_separator<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<SwitchCaseSeparator, ParseError> {
    let token = utils::expect_one_of(stream, T![":", ";"])?;

    Ok(match token.kind {
        T![":"] => SwitchCaseSeparator::Colon(token.span),
        T![";"] => SwitchCaseSeparator::SemiColon(token.span),
        _ => unreachable!(),
    })
}
