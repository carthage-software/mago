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

pub fn parse_switch(stream: &mut TokenStream<'_, '_>) -> Result<Switch, ParseError> {
    Ok(Switch {
        switch: utils::expect_keyword(stream, T!["switch"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        expression: Box::new(parse_expression(stream)?),
        right_parenthesis: utils::expect_span(stream, T![")"])?,
        body: parse_switch_body(stream)?,
    })
}

pub fn parse_switch_body(stream: &mut TokenStream<'_, '_>) -> Result<SwitchBody, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match token.kind {
        T![":"] => SwitchBody::ColonDelimited(parse_switch_colon_delimited_body(stream)?),
        T!["{"] => SwitchBody::BraceDelimited(parse_switch_brace_delimited_body(stream)?),
        _ => {
            return Err(utils::unexpected(stream, Some(token), T![":", "{"]));
        }
    })
}

pub fn parse_switch_brace_delimited_body(
    stream: &mut TokenStream<'_, '_>,
) -> Result<SwitchBraceDelimitedBody, ParseError> {
    let left_brace = utils::expect_span(stream, T!["{"])?;
    let optional_terminator = parse_optional_terminator(stream)?;
    let cases = parse_switch_cases(stream)?;
    let right_brace = utils::expect_span(stream, T!["}"])?;

    Ok(SwitchBraceDelimitedBody { left_brace, optional_terminator, cases, right_brace })
}

pub fn parse_switch_colon_delimited_body(
    stream: &mut TokenStream<'_, '_>,
) -> Result<SwitchColonDelimitedBody, ParseError> {
    Ok(SwitchColonDelimitedBody {
        colon: utils::expect_span(stream, T![":"])?,
        optional_terminator: parse_optional_terminator(stream)?,
        cases: parse_switch_cases(stream)?,
        end_switch: utils::expect_keyword(stream, T!["endswitch"])?,
        terminator: parse_terminator(stream)?,
    })
}

pub fn parse_switch_cases(stream: &mut TokenStream<'_, '_>) -> Result<Sequence<SwitchCase>, ParseError> {
    let mut cases = vec![];
    loop {
        if matches!(utils::peek(stream)?.kind, T!["endswitch" | "}"]) {
            break;
        }

        cases.push(parse_switch_case(stream)?);
    }

    Ok(Sequence::new(cases))
}

pub fn parse_switch_case(stream: &mut TokenStream<'_, '_>) -> Result<SwitchCase, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T!["default"] => SwitchCase::Default(parse_switch_default_case(stream)?),
        _ => SwitchCase::Expression(parse_switch_expression_case(stream)?),
    })
}

pub fn parse_switch_expression_case(stream: &mut TokenStream<'_, '_>) -> Result<SwitchExpressionCase, ParseError> {
    Ok(SwitchExpressionCase {
        case: utils::expect_keyword(stream, T!["case"])?,
        expression: Box::new(parse_expression(stream)?),
        separator: parse_switch_case_separator(stream)?,
        statements: parse_switch_statements(stream)?,
    })
}

pub fn parse_switch_default_case(stream: &mut TokenStream<'_, '_>) -> Result<SwitchDefaultCase, ParseError> {
    Ok(SwitchDefaultCase {
        default: utils::expect_keyword(stream, T!["default"])?,
        separator: parse_switch_case_separator(stream)?,
        statements: parse_switch_statements(stream)?,
    })
}

pub fn parse_switch_statements(stream: &mut TokenStream<'_, '_>) -> Result<Sequence<Statement>, ParseError> {
    let mut statements = vec![];
    loop {
        if matches!(utils::peek(stream)?.kind, T!["case" | "default" | "endswitch" | "}"]) {
            break;
        }

        statements.push(parse_statement(stream)?);
    }

    Ok(Sequence::new(statements))
}

pub fn parse_switch_case_separator(stream: &mut TokenStream<'_, '_>) -> Result<SwitchCaseSeparator, ParseError> {
    let token = utils::expect_one_of(stream, T![":", ";"])?;

    Ok(match token.kind {
        T![":"] => SwitchCaseSeparator::Colon(token.span),
        T![";"] => SwitchCaseSeparator::SemiColon(token.span),
        _ => unreachable!(),
    })
}
