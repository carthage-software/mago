use crate::T;
use crate::ast::ast::DirectVariable;
use crate::ast::ast::IndirectVariable;
use crate::ast::ast::NestedVariable;
use crate::ast::ast::Variable;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
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

        Ok(DirectVariable { span: token.span, name: token.value })
    }

    pub(crate) fn parse_indirect_variable(&mut self) -> Result<IndirectVariable<'arena>, ParseError> {
        let within_indirect_variable = self.state.within_indirect_variable;

        Ok(IndirectVariable {
            dollar_left_brace: self.stream.eat(T!["${"])?.span,
            expression: {
                self.state.within_indirect_variable = true;
                let expr = self.parse_expression()?;
                self.state.within_indirect_variable = within_indirect_variable;

                self.arena.alloc(expr)
            },
            right_brace: self.stream.eat(T!["}"])?.span,
        })
    }

    pub(crate) fn parse_nested_variable(&mut self) -> Result<NestedVariable<'arena>, ParseError> {
        Ok(NestedVariable {
            dollar: self.stream.eat(T!["$"])?.span,
            variable: self.arena.alloc(self.parse_variable()?),
        })
    }
}
