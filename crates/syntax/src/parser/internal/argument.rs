use crate::T;
use crate::ast::ast::Argument;
use crate::ast::ast::ArgumentList;
use crate::ast::ast::NamedArgument;
use crate::ast::ast::NamedPlaceholderArgument;
use crate::ast::ast::PartialArgument;
use crate::ast::ast::PartialArgumentList;
use crate::ast::ast::PlaceholderArgument;
use crate::ast::ast::PositionalArgument;
use crate::ast::ast::VariadicPlaceholderArgument;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_optional_argument_list(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<ArgumentList<'arena>>, ParseError> {
        if let Some(T!["("]) = stream.lookahead(0)?.map(|t| t.kind) {
            Ok(Some(self.parse_argument_list(stream)?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn parse_argument_list(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ArgumentList<'arena>, ParseError> {
        let result = self.parse_comma_separated_sequence(stream, T!["("], T![")"], |p, s| p.parse_argument(s))?;

        Ok(ArgumentList { left_parenthesis: result.open, arguments: result.sequence, right_parenthesis: result.close })
    }

    pub(crate) fn parse_argument(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Argument<'arena>, ParseError> {
        let current = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
        if current.kind.is_identifier_maybe_reserved() && matches!(stream.lookahead(1)?.map(|t| t.kind), Some(T![":"]))
        {
            return Ok(Argument::Named(NamedArgument {
                name: self.parse_local_identifier(stream)?,
                colon: stream.consume()?.span,
                value: self.parse_expression(stream)?,
            }));
        }

        Ok(Argument::Positional(PositionalArgument {
            ellipsis: if stream.is_at(T!["..."])? { Some(stream.eat(T!["..."])?.span) } else { None },
            value: self.parse_expression(stream)?,
        }))
    }

    pub(crate) fn parse_partial_argument_list(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<PartialArgumentList<'arena>, ParseError> {
        let result =
            self.parse_comma_separated_sequence(stream, T!["("], T![")"], |p, s| p.parse_partial_argument(s))?;

        Ok(PartialArgumentList {
            left_parenthesis: result.open,
            arguments: result.sequence,
            right_parenthesis: result.close,
        })
    }

    pub(crate) fn parse_partial_argument(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<PartialArgument<'arena>, ParseError> {
        let current = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        if current.kind == T!["?"] {
            return Ok(PartialArgument::Placeholder(PlaceholderArgument { span: stream.consume()?.span }));
        }

        if current.kind == T!["..."] {
            let next = stream.lookahead(1)?;
            match next.map(|t| t.kind) {
                Some(crate::token::TokenKind::Comma | crate::token::TokenKind::RightParenthesis) | None => {
                    return Ok(PartialArgument::VariadicPlaceholder(VariadicPlaceholderArgument {
                        span: stream.consume()?.span,
                    }));
                }
                _ => {}
            }
        }

        if current.kind.is_identifier_maybe_reserved() && matches!(stream.lookahead(1)?.map(|t| t.kind), Some(T![":"]))
        {
            let name = self.parse_local_identifier(stream)?;
            let colon = stream.consume()?.span;

            if stream.is_at(T!["?"])? {
                let question_mark = stream.eat(T!["?"])?.span;
                return Ok(PartialArgument::NamedPlaceholder(NamedPlaceholderArgument { name, colon, question_mark }));
            }

            return Ok(PartialArgument::Named(NamedArgument { name, colon, value: self.parse_expression(stream)? }));
        }

        Ok(PartialArgument::Positional(PositionalArgument {
            ellipsis: if stream.is_at(T!["..."])? { Some(stream.eat(T!["..."])?.span) } else { None },
            value: self.parse_expression(stream)?,
        }))
    }
}
