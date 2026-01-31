use crate::T;
use crate::ast::ast::Declare;
use crate::ast::ast::DeclareBody;
use crate::ast::ast::DeclareColonDelimitedBody;
use crate::ast::ast::DeclareItem;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_declare(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Declare<'arena>, ParseError> {
        Ok(Declare {
            declare: self.expect_keyword(stream, T!["declare"])?,
            left_parenthesis: stream.eat(T!["("])?.span,
            items: {
                let mut items = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if let Some(T![")"]) = stream.lookahead(0)?.map(|t| t.kind) {
                        break;
                    }

                    items.push(self.parse_declare_item(stream)?);

                    if let Some(T![","]) = stream.lookahead(0)?.map(|t| t.kind) {
                        commas.push(stream.consume()?);
                    } else {
                        break;
                    }
                }

                TokenSeparatedSequence::new(items, commas)
            },
            right_parenthesis: stream.eat(T![")"])?.span,
            body: self.parse_declare_body(stream)?,
        })
    }

    pub(crate) fn parse_declare_item(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<DeclareItem<'arena>, ParseError> {
        Ok(DeclareItem {
            name: self.parse_local_identifier(stream)?,
            equal: stream.eat(T!["="])?.span,
            value: self.parse_expression(stream)?,
        })
    }

    pub(crate) fn parse_declare_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<DeclareBody<'arena>, ParseError> {
        let next = stream.lookahead(0)?;

        Ok(match next.map(|t| t.kind) {
            Some(T![":"]) => DeclareBody::ColonDelimited(self.parse_declare_colon_delimited_body(stream)?),
            _ => DeclareBody::Statement(self.arena.alloc(self.parse_statement(stream)?)),
        })
    }

    pub(crate) fn parse_declare_colon_delimited_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<DeclareColonDelimitedBody<'arena>, ParseError> {
        Ok(DeclareColonDelimitedBody {
            colon: stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if let Some(T!["enddeclare"]) = stream.lookahead(0)?.map(|t| t.kind) {
                        break;
                    }

                    statements.push(self.parse_statement(stream)?);
                }
                Sequence::new(statements)
            },
            end_declare: self.expect_keyword(stream, T!["enddeclare"])?,
            terminator: self.parse_terminator(stream)?,
        })
    }
}
