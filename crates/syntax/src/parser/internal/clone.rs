use crate::T;
use crate::ast::ast::Call;
use crate::ast::ast::Clone;
use crate::ast::ast::Expression;
use crate::ast::ast::FunctionCall;
use crate::ast::ast::FunctionPartialApplication;
use crate::ast::ast::Identifier;
use crate::ast::ast::LocalIdentifier;
use crate::ast::ast::Parenthesized;
use crate::ast::ast::PartialApplication;
use crate::ast::ast::PartialArgument;
use crate::ast::ast::PartialArgumentList;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::argument::parse_partial_argument;
use crate::parser::internal::expression::parse_expression_with_precedence;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;
use crate::token::Precedence;
use crate::token::TokenKind;

/// Parses a `clone` expression, handling the syntactic ambiguity introduced in PHP 8.5.
///
/// PHP 8.5 allows `clone` to be used like a function (e.g., `clone($foo, $bar)`). This
/// creates an ambiguity with the older syntax `clone ($foo)`, which should be parsed as
/// a `clone` expression operating on a parenthesized expression, not a function call.
///
/// This function resolves the ambiguity by looking ahead after the first argument. If the
/// next token is not a comma and the argument is a simple positional one, it assumes
/// the legacy `clone (expr)` structure. Otherwise, it parses the expression as a
/// standard function call.
pub fn parse_ambiguous_clone_expression<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Expression<'arena>, ParseError> {
    let clone = utils::expect_keyword(stream, T!["clone"])?;
    if utils::peek(stream)?.kind != TokenKind::LeftParenthesis {
        return Ok(Expression::Clone(Clone {
            clone,
            object: {
                let object = parse_expression_with_precedence(stream, Precedence::Clone)?;

                stream.alloc(object)
            },
        }));
    }

    let left_parenthesis = utils::expect_span(stream, T!["("])?;

    let mut arguments = stream.new_vec();
    let mut commas = stream.new_vec();
    loop {
        let next = utils::peek(stream)?;
        if next.kind == T![")"] {
            break;
        }

        arguments.push(parse_partial_argument(stream)?);

        let next = utils::peek(stream)?;
        if next.kind == T![","] {
            commas.push(utils::expect_any(stream)?);
        } else {
            break;
        }
    }

    let partial_args = PartialArgumentList {
        left_parenthesis,
        arguments: TokenSeparatedSequence::new(arguments, commas),
        right_parenthesis: utils::expect_span(stream, T![")"])?,
    };

    if partial_args.has_placeholders() {
        return Ok(Expression::PartialApplication(PartialApplication::Function(FunctionPartialApplication {
            function: stream.alloc(Expression::Identifier(Identifier::Local(LocalIdentifier {
                span: clone.span,
                value: clone.value,
            }))),
            argument_list: partial_args,
        })));
    }

    let is_function_call = partial_args.arguments.len() > 1 || {
        matches!(
            partial_args.arguments.first(),
            Some(PartialArgument::Positional(arg)) if arg.ellipsis.is_some()
        )
    };

    if is_function_call {
        return Ok(Expression::Call(Call::Function(FunctionCall {
            function: stream.alloc(Expression::Identifier(Identifier::Local(LocalIdentifier {
                span: clone.span,
                value: clone.value,
            }))),
            argument_list: partial_args.into_argument_list(stream.arena()),
        })));
    }

    let cloned_expression = match partial_args.arguments.into_iter().next() {
        Some(PartialArgument::Positional(arg)) => arg.value,
        Some(PartialArgument::Named(arg)) => arg.value,
        _ => unreachable!("Should have at least one argument"),
    };

    Ok(Expression::Clone(Clone {
        clone,
        object: {
            let object = Expression::Parenthesized(Parenthesized {
                left_parenthesis: partial_args.left_parenthesis,
                expression: stream.alloc(cloned_expression),
                right_parenthesis: partial_args.right_parenthesis,
            });

            stream.alloc(object)
        },
    }))
}
