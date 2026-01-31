use crate::T;
use crate::ast::ast::AttributeList;
use crate::ast::ast::Closure;
use crate::ast::ast::ClosureUseClause;
use crate::ast::ast::ClosureUseClauseVariable;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_closure_with_attributes(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Closure<'arena>, ParseError> {
        Ok(Closure {
            attribute_lists: attributes,
            r#static: self.maybe_expect_keyword(stream, T!["static"])?,
            function: self.expect_keyword(stream, T!["function"])?,
            ampersand: if stream.is_at(T!["&"])? { Some(stream.eat(T!["&"])?.span) } else { None },
            parameter_list: self.parse_function_like_parameter_list(stream)?,
            use_clause: self.parse_optional_closure_use_clause(stream)?,
            return_type_hint: self.parse_optional_function_like_return_type_hint(stream)?,
            body: self.parse_block(stream)?,
        })
    }

    fn parse_optional_closure_use_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<ClosureUseClause<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["use"]) => Some(self.parse_closure_use_clause(stream)?),
            _ => None,
        })
    }

    fn parse_closure_use_clause(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ClosureUseClause<'arena>, ParseError> {
        let r#use = self.expect_keyword(stream, T!["use"])?;
        let result = self
            .parse_comma_separated_sequence(stream, T!["("], T![")"], |p, s| p.parse_closure_use_clause_variable(s))?;

        Ok(ClosureUseClause {
            r#use,
            left_parenthesis: result.open,
            variables: result.sequence,
            right_parenthesis: result.close,
        })
    }

    fn parse_closure_use_clause_variable(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ClosureUseClauseVariable<'arena>, ParseError> {
        Ok(ClosureUseClauseVariable {
            ampersand: if stream.is_at(T!["&"])? { Some(stream.eat(T!["&"])?.span) } else { None },
            variable: self.parse_direct_variable(stream)?,
        })
    }
}
