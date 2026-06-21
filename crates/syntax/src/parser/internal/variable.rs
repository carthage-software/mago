use mago_allocator::prelude::*;
use mago_database::file::HasFileId;

use crate::T;
use crate::cst::cst::ArrayAccess;
use crate::cst::cst::DirectVariable;
use crate::cst::cst::Expression;
use crate::cst::cst::Identifier;
use crate::cst::cst::IndirectVariable;
use crate::cst::cst::LocalIdentifier;
use crate::cst::cst::NestedVariable;
use crate::cst::cst::Variable;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_variable(&mut self) -> Result<Variable<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        Ok(match &token.kind {
            T!["$variable"] => Variable::Direct(self.parse_direct_variable()?),
            T!["${"] => Variable::Indirect(self.parse_indirect_variable()?),
            T!["$"] => Variable::Nested(self.parse_nested_variable()?),
            _ => return Err(self.stream.unexpected(Some(token), T!["$variable", "${", "$"])),
        })
    }

    pub(crate) fn parse_direct_variable(&mut self) -> Result<DirectVariable<'arena>, ParseError> {
        let token = self.stream.eat(T!["$variable"])?;

        Ok(DirectVariable { span: token.span_for(self.stream.file_id()), name: token.value })
    }

    pub(crate) fn parse_indirect_variable(&mut self) -> Result<IndirectVariable<'arena>, ParseError> {
        let dollar_left_brace = self.stream.eat_span(T!["${"])?;

        let expression = if matches!(self.stream.peek_kind(0)?, Some(T![StringVariableName])) {
            let label = self.stream.eat(T![StringVariableName])?;
            let name = self.arena.alloc(Expression::Identifier(Identifier::Local(LocalIdentifier {
                span: label.span_for(self.stream.file_id()),
                value: label.value,
            })));

            if matches!(self.stream.peek_kind(0)?, Some(T!["["])) {
                let left_bracket = self.stream.eat_span(T!["["])?;
                let index = self.parse_expression()?;
                let right_bracket = self.stream.eat_span(T!["]"])?;

                self.arena.alloc(Expression::ArrayAccess(ArrayAccess {
                    array: name,
                    left_bracket,
                    index,
                    right_bracket,
                }))
            } else {
                name
            }
        } else {
            self.parse_expression()?
        };

        Ok(IndirectVariable { dollar_left_brace, expression, right_brace: self.stream.eat_span(T!["}"])? })
    }

    pub(crate) fn parse_nested_variable(&mut self) -> Result<NestedVariable<'arena>, ParseError> {
        Ok(NestedVariable {
            dollar: self.stream.eat_span(T!["$"])?,
            variable: self.arena.alloc(self.parse_variable()?),
        })
    }
}
