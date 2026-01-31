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
use crate::parser::Parser;
use crate::token::Precedence;
use crate::token::TokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
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
    pub(crate) fn parse_ambiguous_clone_expression(&mut self) -> Result<Expression<'arena>, ParseError> {
        let clone = self.expect_keyword(T!["clone"])?;
        let next = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        if next.kind != TokenKind::LeftParenthesis {
            return Ok(Expression::Clone(Clone {
                clone,
                object: self.arena.alloc(self.parse_expression_with_precedence(Precedence::Clone)?),
            }));
        }

        let left_parenthesis = self.stream.eat(T!["("])?.span;

        let mut arguments = self.new_vec();
        let mut commas = self.new_vec();
        loop {
            let next = self.stream.lookahead(0)?;
            if matches!(next.map(|t| t.kind), Some(T![")"])) {
                break;
            }

            arguments.push(self.parse_partial_argument()?);

            if let Some(T![","]) = self.stream.lookahead(0)?.map(|t| t.kind) {
                commas.push(self.stream.consume()?);
            } else {
                break;
            }
        }

        let partial_args = PartialArgumentList {
            left_parenthesis,
            arguments: TokenSeparatedSequence::new(arguments, commas),
            right_parenthesis: self.stream.eat(T![")"])?.span,
        };

        if partial_args.has_placeholders() {
            return Ok(Expression::PartialApplication(PartialApplication::Function(FunctionPartialApplication {
                function: self.arena.alloc(Expression::Identifier(Identifier::Local(LocalIdentifier {
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
                function: self.arena.alloc(Expression::Identifier(Identifier::Local(LocalIdentifier {
                    span: clone.span,
                    value: clone.value,
                }))),
                argument_list: partial_args.into_argument_list(self.arena),
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
                    expression: self.arena.alloc(cloned_expression),
                    right_parenthesis: partial_args.right_parenthesis,
                });

                self.arena.alloc(object)
            },
        }))
    }
}
