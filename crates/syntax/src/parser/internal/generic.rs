use mago_database::file::HasFileId;
use mago_span::Position;
use mago_span::Span;

use crate::T;
use crate::ast::ast::GenericArgumentList;
use crate::ast::ast::GenericParameter;
use crate::ast::ast::GenericParameterBound;
use crate::ast::ast::GenericParameterDefault;
use crate::ast::ast::GenericParameterList;
use crate::ast::ast::GenericVariance;
use crate::ast::ast::Hint;
use crate::ast::ast::Turbofish;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TokenKind;

impl<'arena> Parser<'_, 'arena> {
    /// Returns `true` when the next significant token is a `<` and could
    /// plausibly open a generic argument or parameter list at this position.
    ///
    /// Callers are expected to gate this on PHP-version support; the parser
    /// itself is lenient and lets semantics reject pre-8.6 usage.
    pub(crate) fn is_at_generic_open_angle(&mut self) -> Result<bool, ParseError> {
        Ok(matches!(self.stream.peek_kind(0)?, Some(T!["<"])))
    }

    /// Parse `<T, U, ...>` if the next token is `<`; otherwise `None`.
    pub(crate) fn parse_optional_generic_parameter_list(
        &mut self,
    ) -> Result<Option<GenericParameterList<'arena>>, ParseError> {
        if self.is_at_generic_open_angle()? { Ok(Some(self.parse_generic_parameter_list()?)) } else { Ok(None) }
    }

    /// Parse `<T, U, ...>` given that the next token is `<`.
    ///
    /// Each entry is `[+|-] NAME [: Bound] [= Default]`.
    pub(crate) fn parse_generic_parameter_list(&mut self) -> Result<GenericParameterList<'arena>, ParseError> {
        let less_than = self.stream.eat_span(T!["<"])?;

        let mut items = self.new_vec();
        let mut separators = self.new_vec();
        loop {
            if self.is_at_close_angle()? {
                break;
            }

            match self.parse_generic_parameter() {
                Ok(parameter) => items.push(parameter),
                Err(err) => {
                    self.errors.push(err);
                    break;
                }
            }

            if matches!(self.stream.peek_kind(0)?, Some(T![","])) {
                separators.push(self.stream.consume()?);
            } else {
                break;
            }
        }

        let greater_than = self.eat_close_angle()?;

        Ok(GenericParameterList { less_than, parameters: TokenSeparatedSequence::new(items, separators), greater_than })
    }

    fn parse_generic_parameter(&mut self) -> Result<GenericParameter<'arena>, ParseError> {
        let variance = match self.stream.peek_kind(0)? {
            Some(T!["+"]) => Some(GenericVariance::Covariant(self.stream.eat_span(T!["+"])?)),
            Some(T!["-"]) => Some(GenericVariance::Contravariant(self.stream.eat_span(T!["-"])?)),
            _ => None,
        };

        let name = self.parse_local_identifier()?;

        let bound = if matches!(self.stream.peek_kind(0)?, Some(T![":"])) {
            let colon = self.stream.eat_span(T![":"])?;
            let hint = self.parse_type_hint()?;
            Some(GenericParameterBound { colon, hint })
        } else {
            None
        };

        let default = if matches!(self.stream.peek_kind(0)?, Some(T!["="])) {
            let equals = self.stream.eat_span(T!["="])?;
            let hint = self.parse_type_hint()?;
            Some(GenericParameterDefault { equals, hint })
        } else {
            None
        };

        Ok(GenericParameter { variance, name, bound, default })
    }

    /// Parse `<T, U, ...>` if the next token is `<`; otherwise `None`.
    ///
    /// The type-hint parser handles this inline today, so this helper exists
    /// for upcoming inheritance / `instanceof` / `catch` work that needs to
    /// attach a generic argument list to a class-name reference.
    #[allow(dead_code)]
    pub(crate) fn parse_optional_generic_argument_list(
        &mut self,
    ) -> Result<Option<GenericArgumentList<'arena>>, ParseError> {
        if self.is_at_generic_open_angle()? { Ok(Some(self.parse_generic_argument_list()?)) } else { Ok(None) }
    }

    /// Parse `<T, U, ...>` given that the next token is `<`.
    pub(crate) fn parse_generic_argument_list(&mut self) -> Result<GenericArgumentList<'arena>, ParseError> {
        let less_than = self.stream.eat_span(T!["<"])?;
        let (arguments, greater_than) = self.parse_generic_argument_list_tail()?;
        Ok(GenericArgumentList { less_than, arguments, greater_than })
    }

    /// Parse `::<T, U, ...>` if the next token is `::<`; otherwise `None`.
    pub(crate) fn parse_optional_turbofish(&mut self) -> Result<Option<Turbofish<'arena>>, ParseError> {
        if matches!(self.stream.peek_kind(0)?, Some(T!["::<"])) { Ok(Some(self.parse_turbofish()?)) } else { Ok(None) }
    }

    /// Parse `::<T, U, ...>` given that the next token is `::<`.
    pub(crate) fn parse_turbofish(&mut self) -> Result<Turbofish<'arena>, ParseError> {
        let turbofish = self.stream.eat_span(T!["::<"])?;
        let (arguments, greater_than) = self.parse_generic_argument_list_tail()?;
        Ok(Turbofish { turbofish, arguments, greater_than })
    }

    /// Parse the comma-separated body and closing `>` shared by every
    /// generic-argument list shape (type-use lists and turbofish).
    fn parse_generic_argument_list_tail(
        &mut self,
    ) -> Result<(TokenSeparatedSequence<'arena, Hint<'arena>>, Span), ParseError> {
        let mut items = self.new_vec();
        let mut separators = self.new_vec();
        loop {
            if self.is_at_close_angle()? {
                break;
            }

            match self.parse_type_hint() {
                Ok(hint) => items.push(hint),
                Err(err) => {
                    self.errors.push(err);
                    break;
                }
            }

            if matches!(self.stream.peek_kind(0)?, Some(T![","])) {
                separators.push(self.stream.consume()?);
            } else {
                break;
            }
        }

        let greater_than = self.eat_close_angle()?;
        Ok((TokenSeparatedSequence::new(items, separators), greater_than))
    }

    /// True if the next token closes a generic angle bracket: a real `>` or
    /// the first half of a `>>` (right-shift), or a pending virtual `>`
    /// recorded from a previous split.
    pub(crate) fn is_at_close_angle(&mut self) -> Result<bool, ParseError> {
        if self.state.pending_close_angles > 0 {
            return Ok(true);
        }

        Ok(matches!(self.stream.peek_kind(0)?, Some(T![">"] | TokenKind::RightShift)))
    }

    /// Consume one closing `>` token, splitting `>>` when needed so a nested
    /// generic list can close with the second half on the next call.
    pub(crate) fn eat_close_angle(&mut self) -> Result<Span, ParseError> {
        if self.state.pending_close_angles > 0 {
            self.state.pending_close_angles -= 1;
            let offset = self.state.pending_close_angle_offset;
            let start = Position::new(offset);
            let end = Position::new(offset + 1);
            return Ok(Span::new(self.stream.file_id(), start, end));
        }

        match self.stream.peek_kind(0)? {
            Some(T![">"]) => Ok(self.stream.eat_span(T![">"])?),
            Some(TokenKind::RightShift) => {
                let token = self.stream.consume()?;
                let first_offset = token.start.offset;
                let second_offset = first_offset + 1;
                self.state.pending_close_angles = 1;
                self.state.pending_close_angle_offset = second_offset;
                let start = Position::new(first_offset);
                let end = Position::new(second_offset);
                Ok(Span::new(self.stream.file_id(), start, end))
            }
            _ => {
                let token = self.stream.lookahead(0)?;
                Err(self.stream.unexpected(token, &[T![">"]]))
            }
        }
    }
}
