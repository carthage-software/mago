use crate::T;
use crate::ast::ast::AttributeList;
use crate::ast::ast::Closure;
use crate::ast::ast::ClosureUseClause;
use crate::ast::ast::ClosureUseClauseVariable;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_closure_with_attributes(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Closure<'arena>, ParseError> {
        Ok(Closure {
            attribute_lists: attributes,
            r#static: self.maybe_expect_keyword(T!["static"])?,
            function: self.expect_keyword(T!["function"])?,
            ampersand: if self.stream.is_at(T!["&"])? { Some(self.stream.eat_span(T!["&"])?) } else { None },
            parameter_list: self.parse_function_like_parameter_list()?,
            use_clause: self.parse_optional_closure_use_clause()?,
            return_type_hint: self.parse_optional_function_like_return_type_hint()?,
            body: self.parse_block()?,
        })
    }

    fn parse_optional_closure_use_clause(&mut self) -> Result<Option<ClosureUseClause<'arena>>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T!["use"]) => Some(self.parse_closure_use_clause()?),
            _ => None,
        })
    }

    fn parse_closure_use_clause(&mut self) -> Result<ClosureUseClause<'arena>, ParseError> {
        let r#use = self.expect_keyword(T!["use"])?;
        let result =
            self.parse_comma_separated_sequence(T!["("], T![")"], |p| p.parse_closure_use_clause_variable())?;

        Ok(ClosureUseClause {
            r#use,
            left_parenthesis: result.open,
            variables: result.sequence,
            right_parenthesis: result.close,
        })
    }

    fn parse_closure_use_clause_variable(&mut self) -> Result<ClosureUseClauseVariable<'arena>, ParseError> {
        Ok(ClosureUseClauseVariable {
            ampersand: if self.stream.is_at(T!["&"])? { Some(self.stream.eat_span(T!["&"])?) } else { None },
            variable: self.parse_direct_variable()?,
        })
    }
}
