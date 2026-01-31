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
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_if(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<If<'arena>, ParseError> {
        Ok(If {
            r#if: self.expect_keyword(stream, T!["if"])?,
            left_parenthesis: stream.eat(T!["("])?.span,
            condition: self.arena.alloc(self.parse_expression(stream)?),
            right_parenthesis: stream.eat(T![")"])?.span,
            body: self.parse_if_body(stream)?,
        })
    }

    fn parse_if_body(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<IfBody<'arena>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![":"]) => IfBody::ColonDelimited(self.parse_if_colon_delimited_body(stream)?),
            _ => IfBody::Statement(self.parse_if_statement_body(stream)?),
        })
    }

    fn parse_if_statement_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<IfStatementBody<'arena>, ParseError> {
        Ok(IfStatementBody {
            statement: self.arena.alloc(self.parse_statement(stream)?),
            else_if_clauses: {
                let mut else_if_clauses = self.new_vec();
                while let Some(else_if_clause) = self.parse_optional_if_statement_body_else_if_clause(stream)? {
                    else_if_clauses.push(else_if_clause);
                }

                Sequence::new(else_if_clauses)
            },
            else_clause: self.parse_optional_if_statement_body_else_clause(stream)?,
        })
    }

    fn parse_optional_if_statement_body_else_if_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<IfStatementBodyElseIfClause<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["elseif"]) => Some(self.parse_if_statement_body_else_if_clause(stream)?),
            _ => None,
        })
    }

    fn parse_if_statement_body_else_if_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<IfStatementBodyElseIfClause<'arena>, ParseError> {
        Ok(IfStatementBodyElseIfClause {
            elseif: self.expect_keyword(stream, T!["elseif"])?,
            left_parenthesis: stream.eat(T!["("])?.span,
            condition: self.arena.alloc(self.parse_expression(stream)?),
            right_parenthesis: stream.eat(T![")"])?.span,
            statement: self.arena.alloc(self.parse_statement(stream)?),
        })
    }

    fn parse_optional_if_statement_body_else_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<IfStatementBodyElseClause<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["else"]) => Some(self.parse_if_statement_body_else_clause(stream)?),
            _ => None,
        })
    }

    fn parse_if_statement_body_else_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<IfStatementBodyElseClause<'arena>, ParseError> {
        Ok(IfStatementBodyElseClause {
            r#else: self.expect_keyword(stream, T!["else"])?,
            statement: self.arena.alloc(self.parse_statement(stream)?),
        })
    }

    fn parse_if_colon_delimited_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<IfColonDelimitedBody<'arena>, ParseError> {
        Ok(IfColonDelimitedBody {
            colon: stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["elseif" | "else" | "endif"])) {
                        break;
                    }

                    statements.push(self.parse_statement(stream)?);
                }

                Sequence::new(statements)
            },
            else_if_clauses: {
                let mut else_if_clauses = self.new_vec();
                while let Some(else_if_clause) = self.parse_optional_if_colon_delimited_body_else_if_clause(stream)? {
                    else_if_clauses.push(else_if_clause);
                }

                Sequence::new(else_if_clauses)
            },
            else_clause: self.parse_optional_if_colon_delimited_body_else_clause(stream)?,
            endif: self.expect_keyword(stream, T!["endif"])?,
            terminator: self.parse_terminator(stream)?,
        })
    }

    fn parse_optional_if_colon_delimited_body_else_if_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<IfColonDelimitedBodyElseIfClause<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["elseif"]) => Some(self.parse_if_colon_delimited_body_else_if_clause(stream)?),
            _ => None,
        })
    }

    fn parse_if_colon_delimited_body_else_if_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<IfColonDelimitedBodyElseIfClause<'arena>, ParseError> {
        Ok(IfColonDelimitedBodyElseIfClause {
            r#elseif: self.expect_keyword(stream, T!["elseif"])?,
            left_parenthesis: stream.eat(T!["("])?.span,
            condition: self.arena.alloc(self.parse_expression(stream)?),
            right_parenthesis: stream.eat(T![")"])?.span,
            colon: stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["elseif" | "else" | "endif"])) {
                        break;
                    }

                    statements.push(self.parse_statement(stream)?);
                }

                Sequence::new(statements)
            },
        })
    }

    fn parse_optional_if_colon_delimited_body_else_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<IfColonDelimitedBodyElseClause<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["else"]) => Some(self.parse_if_colon_delimited_body_else_clause(stream)?),
            _ => None,
        })
    }

    fn parse_if_colon_delimited_body_else_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<IfColonDelimitedBodyElseClause<'arena>, ParseError> {
        Ok(IfColonDelimitedBodyElseClause {
            r#else: self.expect_keyword(stream, T!["else"])?,
            colon: stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["endif"])) {
                        break;
                    }

                    statements.push(self.parse_statement(stream)?);
                }
                Sequence::new(statements)
            },
        })
    }
}
