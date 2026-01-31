use crate::T;
use crate::ast::ast::If;
use crate::ast::ast::IfBody;
use crate::ast::ast::IfColonDelimitedBody;
use crate::ast::ast::IfColonDelimitedBodyElseClause;
use crate::ast::ast::IfColonDelimitedBodyElseIfClause;
use crate::ast::ast::IfStatementBody;
use crate::ast::ast::IfStatementBodyElseClause;
use crate::ast::ast::IfStatementBodyElseIfClause;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_if(&mut self) -> Result<If<'arena>, ParseError> {
        Ok(If {
            r#if: self.expect_keyword(T!["if"])?,
            left_parenthesis: self.stream.eat(T!["("])?.span,
            condition: self.arena.alloc(self.parse_expression()?),
            right_parenthesis: self.stream.eat(T![")"])?.span,
            body: self.parse_if_body()?,
        })
    }

    fn parse_if_body(&mut self) -> Result<IfBody<'arena>, ParseError> {
        Ok(match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![":"]) => IfBody::ColonDelimited(self.parse_if_colon_delimited_body()?),
            _ => IfBody::Statement(self.parse_if_statement_body()?),
        })
    }

    fn parse_if_statement_body(&mut self) -> Result<IfStatementBody<'arena>, ParseError> {
        Ok(IfStatementBody {
            statement: self.arena.alloc(self.parse_statement()?),
            else_if_clauses: {
                let mut else_if_clauses = self.new_vec();
                while let Some(else_if_clause) = self.parse_optional_if_statement_body_else_if_clause()? {
                    else_if_clauses.push(else_if_clause);
                }

                Sequence::new(else_if_clauses)
            },
            else_clause: self.parse_optional_if_statement_body_else_clause()?,
        })
    }

    fn parse_optional_if_statement_body_else_if_clause(
        &mut self,
    ) -> Result<Option<IfStatementBodyElseIfClause<'arena>>, ParseError> {
        Ok(match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["elseif"]) => Some(self.parse_if_statement_body_else_if_clause()?),
            _ => None,
        })
    }

    fn parse_if_statement_body_else_if_clause(&mut self) -> Result<IfStatementBodyElseIfClause<'arena>, ParseError> {
        Ok(IfStatementBodyElseIfClause {
            elseif: self.expect_keyword(T!["elseif"])?,
            left_parenthesis: self.stream.eat(T!["("])?.span,
            condition: self.arena.alloc(self.parse_expression()?),
            right_parenthesis: self.stream.eat(T![")"])?.span,
            statement: self.arena.alloc(self.parse_statement()?),
        })
    }

    fn parse_optional_if_statement_body_else_clause(
        &mut self,
    ) -> Result<Option<IfStatementBodyElseClause<'arena>>, ParseError> {
        Ok(match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["else"]) => Some(self.parse_if_statement_body_else_clause()?),
            _ => None,
        })
    }

    fn parse_if_statement_body_else_clause(&mut self) -> Result<IfStatementBodyElseClause<'arena>, ParseError> {
        Ok(IfStatementBodyElseClause {
            r#else: self.expect_keyword(T!["else"])?,
            statement: self.arena.alloc(self.parse_statement()?),
        })
    }

    fn parse_if_colon_delimited_body(&mut self) -> Result<IfColonDelimitedBody<'arena>, ParseError> {
        Ok(IfColonDelimitedBody {
            colon: self.stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["elseif" | "else" | "endif"])) {
                        break;
                    }

                    statements.push(self.parse_statement()?);
                }

                Sequence::new(statements)
            },
            else_if_clauses: {
                let mut else_if_clauses = self.new_vec();
                while let Some(else_if_clause) = self.parse_optional_if_colon_delimited_body_else_if_clause()? {
                    else_if_clauses.push(else_if_clause);
                }

                Sequence::new(else_if_clauses)
            },
            else_clause: self.parse_optional_if_colon_delimited_body_else_clause()?,
            endif: self.expect_keyword(T!["endif"])?,
            terminator: self.parse_terminator()?,
        })
    }

    fn parse_optional_if_colon_delimited_body_else_if_clause(
        &mut self,
    ) -> Result<Option<IfColonDelimitedBodyElseIfClause<'arena>>, ParseError> {
        Ok(match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["elseif"]) => Some(self.parse_if_colon_delimited_body_else_if_clause()?),
            _ => None,
        })
    }

    fn parse_if_colon_delimited_body_else_if_clause(
        &mut self,
    ) -> Result<IfColonDelimitedBodyElseIfClause<'arena>, ParseError> {
        Ok(IfColonDelimitedBodyElseIfClause {
            r#elseif: self.expect_keyword(T!["elseif"])?,
            left_parenthesis: self.stream.eat(T!["("])?.span,
            condition: self.arena.alloc(self.parse_expression()?),
            right_parenthesis: self.stream.eat(T![")"])?.span,
            colon: self.stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["elseif" | "else" | "endif"])) {
                        break;
                    }

                    statements.push(self.parse_statement()?);
                }

                Sequence::new(statements)
            },
        })
    }

    fn parse_optional_if_colon_delimited_body_else_clause(
        &mut self,
    ) -> Result<Option<IfColonDelimitedBodyElseClause<'arena>>, ParseError> {
        Ok(match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["else"]) => Some(self.parse_if_colon_delimited_body_else_clause()?),
            _ => None,
        })
    }

    fn parse_if_colon_delimited_body_else_clause(
        &mut self,
    ) -> Result<IfColonDelimitedBodyElseClause<'arena>, ParseError> {
        Ok(IfColonDelimitedBodyElseClause {
            r#else: self.expect_keyword(T!["else"])?,
            colon: self.stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["endif"])) {
                        break;
                    }

                    statements.push(self.parse_statement()?);
                }
                Sequence::new(statements)
            },
        })
    }
}
