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

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_optional_argument_list(&mut self) -> Result<Option<ArgumentList<'arena>>, ParseError> {
        if let Some(T!["("]) = self.stream.lookahead(0)?.map(|t| t.kind) {
            Ok(Some(self.parse_argument_list()?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn parse_argument_list(&mut self) -> Result<ArgumentList<'arena>, ParseError> {
        let result = self.parse_comma_separated_sequence(T!["("], T![")"], |p| p.parse_argument())?;

        Ok(ArgumentList { left_parenthesis: result.open, arguments: result.sequence, right_parenthesis: result.close })
    }

    pub(crate) fn parse_argument(&mut self) -> Result<Argument<'arena>, ParseError> {
        let current = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        if current.kind.is_identifier_maybe_reserved()
            && matches!(self.stream.lookahead(1)?.map(|t| t.kind), Some(T![":"]))
        {
            return Ok(Argument::Named(NamedArgument {
                name: self.parse_local_identifier()?,
                colon: self.stream.consume()?.span,
                value: self.parse_expression()?,
            }));
        }

        Ok(Argument::Positional(PositionalArgument {
            ellipsis: if self.stream.is_at(T!["..."])? { Some(self.stream.eat(T!["..."])?.span) } else { None },
            value: self.parse_expression()?,
        }))
    }

    pub(crate) fn parse_partial_argument_list(&mut self) -> Result<PartialArgumentList<'arena>, ParseError> {
        let result = self.parse_comma_separated_sequence(T!["("], T![")"], |p| p.parse_partial_argument())?;

        Ok(PartialArgumentList {
            left_parenthesis: result.open,
            arguments: result.sequence,
            right_parenthesis: result.close,
        })
    }

    pub(crate) fn parse_partial_argument(&mut self) -> Result<PartialArgument<'arena>, ParseError> {
        let current = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        if current.kind == T!["?"] {
            return Ok(PartialArgument::Placeholder(PlaceholderArgument { span: self.stream.consume()?.span }));
        }

        if current.kind == T!["..."] {
            let next = self.stream.lookahead(1)?;
            match next.map(|t| t.kind) {
                Some(crate::token::TokenKind::Comma | crate::token::TokenKind::RightParenthesis) | None => {
                    return Ok(PartialArgument::VariadicPlaceholder(VariadicPlaceholderArgument {
                        span: self.stream.consume()?.span,
                    }));
                }
                _ => {}
            }
        }

        if current.kind.is_identifier_maybe_reserved()
            && matches!(self.stream.lookahead(1)?.map(|t| t.kind), Some(T![":"]))
        {
            let name = self.parse_local_identifier()?;
            let colon = self.stream.consume()?.span;

            if self.stream.is_at(T!["?"])? {
                let question_mark = self.stream.eat(T!["?"])?.span;
                return Ok(PartialArgument::NamedPlaceholder(NamedPlaceholderArgument { name, colon, question_mark }));
            }

            return Ok(PartialArgument::Named(NamedArgument { name, colon, value: self.parse_expression()? }));
        }

        Ok(PartialArgument::Positional(PositionalArgument {
            ellipsis: if self.stream.is_at(T!["..."])? { Some(self.stream.eat(T!["..."])?.span) } else { None },
            value: self.parse_expression()?,
        }))
    }
}
