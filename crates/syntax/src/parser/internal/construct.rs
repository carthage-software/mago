use crate::T;
use crate::ast::ast::Construct;
use crate::ast::ast::DieConstruct;
use crate::ast::ast::EmptyConstruct;
use crate::ast::ast::EvalConstruct;
use crate::ast::ast::ExitConstruct;
use crate::ast::ast::IncludeConstruct;
use crate::ast::ast::IncludeOnceConstruct;
use crate::ast::ast::IssetConstruct;
use crate::ast::ast::PrintConstruct;
use crate::ast::ast::RequireConstruct;
use crate::ast::ast::RequireOnceConstruct;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::Precedence;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_construct(&mut self) -> Result<Construct<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        Ok(match token.kind {
            T!["isset"] => {
                let isset = self.expect_keyword(T!["isset"])?;
                let result = self.parse_comma_separated_sequence(T!["("], T![")"], |p| p.parse_expression())?;

                Construct::Isset(IssetConstruct {
                    isset,
                    left_parenthesis: result.open,
                    values: result.sequence,
                    right_parenthesis: result.close,
                })
            }
            T!["empty"] => Construct::Empty(EmptyConstruct {
                empty: self.expect_keyword(T!["empty"])?,
                left_parenthesis: self.stream.eat(T!["("])?.span,
                value: self.arena.alloc(self.parse_expression()?),
                right_parenthesis: self.stream.eat(T![")"])?.span,
            }),
            T!["eval"] => Construct::Eval(EvalConstruct {
                eval: self.expect_keyword(T!["eval"])?,
                left_parenthesis: self.stream.eat(T!["("])?.span,
                value: self.arena.alloc(self.parse_expression()?),
                right_parenthesis: self.stream.eat(T![")"])?.span,
            }),
            T!["print"] => Construct::Print(PrintConstruct {
                print: self.expect_keyword(T!["print"])?,
                value: self.arena.alloc(self.parse_expression_with_precedence(Precedence::Print)?),
            }),
            T!["require"] => Construct::Require(RequireConstruct {
                require: self.expect_any_keyword()?,
                value: self.arena.alloc(self.parse_expression()?),
            }),
            T!["require_once"] => Construct::RequireOnce(RequireOnceConstruct {
                require_once: self.expect_any_keyword()?,
                value: self.arena.alloc(self.parse_expression()?),
            }),
            T!["include"] => Construct::Include(IncludeConstruct {
                include: self.expect_any_keyword()?,
                value: self.arena.alloc(self.parse_expression()?),
            }),
            T!["include_once"] => Construct::IncludeOnce(IncludeOnceConstruct {
                include_once: self.expect_any_keyword()?,
                value: self.arena.alloc(self.parse_expression()?),
            }),
            T!["exit"] => Construct::Exit(ExitConstruct {
                exit: self.expect_any_keyword()?,
                arguments: self.parse_optional_argument_list()?,
            }),
            T!["die"] => Construct::Die(DieConstruct {
                die: self.expect_any_keyword()?,
                arguments: self.parse_optional_argument_list()?,
            }),
            _ => {
                return Err(self.stream.unexpected(
                    Some(token),
                    T![
                        "isset",
                        "empty",
                        "eval",
                        "include",
                        "include_once",
                        "require",
                        "require_once",
                        "print",
                        "exit",
                        "die"
                    ],
                ));
            }
        })
    }
}
