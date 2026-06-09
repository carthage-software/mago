use mago_allocator::prelude::*;
use mago_database::file::HasFileId;
use mago_span::Span;

use crate::ast::Print;
use crate::ast::Sequence;
use crate::ast::Statement;
use crate::ast::Text;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;

/// Trait used by block parsers to recognise their closing `{% endX %}` tag.
pub trait Terminator {
    fn is_end(&self, name: &[u8]) -> bool;
}

pub struct NoTerminator;

impl Terminator for NoTerminator {
    fn is_end(&self, _name: &[u8]) -> bool {
        false
    }
}

pub struct BlockTerminator {
    pub names: &'static [&'static [u8]],
}

impl Terminator for BlockTerminator {
    fn is_end(&self, name: &[u8]) -> bool {
        self.names.contains(&name)
    }
}

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    /// Parse a sequence of top-level statements until `terminator` matches
    /// or EOF is reached.
    ///
    /// Block bodies recurse back through here (`parse_statements` ->
    /// `parse_tag` -> e.g. `parse_if` -> `parse_statements`), so the same
    /// recursion-depth guard used for expressions bounds block nesting and
    /// keeps pathologically nested input (e.g. `{% if x %}` repeated) from
    /// overflowing the native stack.
    pub(crate) fn parse_statements<T>(
        &mut self,
        terminator: &T,
    ) -> Result<Sequence<'arena, Statement<'arena>>, ParseError<'arena>>
    where
        T: Terminator,
    {
        self.state.recursion_depth += 1;
        if self.state.recursion_depth > crate::parser::MAX_RECURSION_DEPTH {
            self.state.recursion_depth -= 1;
            let position = self.stream.current_position();
            return Err(ParseError::RecursionLimitExceeded(Span::new(self.stream.file_id(), position, position)));
        }

        let result = self.parse_statements_inner(terminator);
        self.state.recursion_depth -= 1;
        result
    }

    fn parse_statements_inner<T>(
        &mut self,
        terminator: &T,
    ) -> Result<Sequence<'arena, Statement<'arena>>, ParseError<'arena>>
    where
        T: Terminator,
    {
        let mut statements = self.new_vec();
        while let Some(token) = self.stream.lookahead(0)? {
            match token.kind {
                TwigTokenKind::RawText => {
                    let t = self.stream.consume()?;
                    statements.push(Statement::Text(Text { value: t.value, span: self.stream.span_of(&t) }));
                }
                kind if kind.is_open_variable() => {
                    let start_tok = self.stream.consume()?;
                    let open_variable = self.stream.span_of(&start_tok);
                    let expression = self.parse_expression()?;
                    let close_variable = self.stream.expect_variable_end()?;
                    statements.push(Statement::Print(Print { open_variable, expression, close_variable }));
                }
                kind if kind.is_open_block() => {
                    let Some(name_tok) = self.stream.lookahead(1)? else {
                        return Err(self.stream.unexpected(None, &[TwigTokenKind::Name]));
                    };
                    if name_tok.kind != TwigTokenKind::Name {
                        return Err(ParseError::UnexpectedToken(
                            b"a block tag must start with a tag name",
                            self.stream.span_of(&name_tok),
                        ));
                    }
                    if terminator.is_end(name_tok.value) {
                        // Leave the `{%` + keyword for the caller to consume.
                        break;
                    }

                    let open_tag = self.stream.consume()?;
                    let keyword_tok = self.stream.consume()?;
                    let statement = self.parse_tag(open_tag, keyword_tok)?;
                    statements.push(statement);
                }
                TwigTokenKind::VerbatimText => {
                    // Should never appear at the statement level outside verbatim handling.
                    self.stream.consume()?;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.arena.alloc_slice_copy(
                            format!("unexpected token {:?} at statement level", token.kind).as_bytes(),
                        ),
                        self.stream.span_of(&token),
                    ));
                }
            }
        }

        Ok(Sequence::new(statements))
    }
}
