use crate::T;
use crate::ast::ast::Declare;
use crate::ast::ast::DeclareBody;
use crate::ast::ast::DeclareColonDelimitedBody;
use crate::ast::ast::DeclareItem;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_declare(&mut self) -> Result<Declare<'arena>, ParseError> {
        Ok(Declare {
            declare: self.expect_keyword(T!["declare"])?,
            left_parenthesis: self.stream.eat(T!["("])?.span,
            items: {
                let mut items = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if let Some(T![")"]) = self.stream.lookahead(0)?.map(|t| t.kind) {
                        break;
                    }

                    items.push(self.parse_declare_item()?);

                    if let Some(T![","]) = self.stream.lookahead(0)?.map(|t| t.kind) {
                        commas.push(self.stream.consume()?);
                    } else {
                        break;
                    }
                }

                TokenSeparatedSequence::new(items, commas)
            },
            right_parenthesis: self.stream.eat(T![")"])?.span,
            body: self.parse_declare_body()?,
        })
    }

    pub(crate) fn parse_declare_item(&mut self) -> Result<DeclareItem<'arena>, ParseError> {
        Ok(DeclareItem {
            name: self.parse_local_identifier()?,
            equal: self.stream.eat(T!["="])?.span,
            value: self.parse_expression()?,
        })
    }

    pub(crate) fn parse_declare_body(&mut self) -> Result<DeclareBody<'arena>, ParseError> {
        let next = self.stream.lookahead(0)?;

        Ok(match next.map(|t| t.kind) {
            Some(T![":"]) => DeclareBody::ColonDelimited(self.parse_declare_colon_delimited_body()?),
            _ => DeclareBody::Statement(self.arena.alloc(self.parse_statement()?)),
        })
    }

    pub(crate) fn parse_declare_colon_delimited_body(
        &mut self,
    ) -> Result<DeclareColonDelimitedBody<'arena>, ParseError> {
        Ok(DeclareColonDelimitedBody {
            colon: self.stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if let Some(T!["enddeclare"]) = self.stream.lookahead(0)?.map(|t| t.kind) {
                        break;
                    }

                    let position_before = self.stream.current_position();
                    statements.push(self.parse_statement()?);
                    if self.stream.current_position() == position_before {
                        if let Ok(Some(token)) = self.stream.lookahead(0) {
                            self.errors.push(self.stream.unexpected(Some(token), &[]));
                            let _ = self.stream.consume();
                        } else {
                            break;
                        }
                    }
                }
                Sequence::new(statements)
            },
            end_declare: self.expect_keyword(T!["enddeclare"])?,
            terminator: self.parse_terminator()?,
        })
    }
}
