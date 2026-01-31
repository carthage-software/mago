use crate::T;
use crate::ast::ast::Unset;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_unset(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Unset<'arena>, ParseError> {
        let unset = self.expect_keyword(stream, T!["unset"])?;
        let result = self.parse_comma_separated_sequence(stream, T!["("], T![")"], |p, s| p.parse_expression(s))?;
        let terminator = self.parse_terminator(stream)?;

        Ok(Unset {
            unset,
            left_parenthesis: result.open,
            values: result.sequence,
            right_parenthesis: result.close,
            terminator,
        })
    }
}
