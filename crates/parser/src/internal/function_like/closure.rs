use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::block::parse_block;
use crate::internal::function_like::parameter::parse_function_like_parameter_list;
use crate::internal::function_like::r#return::parse_optional_function_like_return_type_hint;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;
use crate::internal::variable::parse_direct_variable;

pub fn parse_closure_with_attributes(
    stream: &mut TokenStream<'_, '_>,
    attributes: Sequence<AttributeList>,
) -> Result<Closure, ParseError> {
    Ok(Closure {
        attribute_lists: attributes,
        r#static: utils::maybe_expect_keyword(stream, T!["static"])?,
        function: utils::expect_keyword(stream, T!["function"])?,
        ampersand: utils::maybe_expect(stream, T!["&"])?.map(|t| t.span),
        parameter_list: parse_function_like_parameter_list(stream)?,
        use_clause: parse_optional_closure_use_clause(stream)?,
        return_type_hint: parse_optional_function_like_return_type_hint(stream)?,
        body: parse_block(stream)?,
    })
}

pub fn parse_optional_closure_use_clause(
    stream: &mut TokenStream<'_, '_>,
) -> Result<Option<ClosureUseClause>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["use"]) => Some(parse_closure_use_clause(stream)?),
        _ => None,
    })
}

pub fn parse_closure_use_clause(stream: &mut TokenStream<'_, '_>) -> Result<ClosureUseClause, ParseError> {
    Ok(ClosureUseClause {
        r#use: utils::expect_keyword(stream, T!["use"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        variables: {
            let mut variables = Vec::new();
            let mut commas = Vec::new();
            loop {
                let token = utils::peek(stream)?;
                if T![")"] == token.kind {
                    break;
                }

                variables.push(parse_closure_use_clause_variable(stream)?);

                match utils::maybe_expect(stream, T![","])? {
                    Some(comma) => {
                        commas.push(comma);
                    }
                    None => break,
                }
            }

            TokenSeparatedSequence::new(variables, commas)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
    })
}

pub fn parse_closure_use_clause_variable(
    stream: &mut TokenStream<'_, '_>,
) -> Result<ClosureUseClauseVariable, ParseError> {
    Ok(ClosureUseClauseVariable {
        ampersand: utils::maybe_expect(stream, T!["&"])?.map(|t| t.span),
        variable: parse_direct_variable(stream)?,
    })
}
