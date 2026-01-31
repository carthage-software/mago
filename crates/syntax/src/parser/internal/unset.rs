use crate::T;
use crate::ast::ast::Unset;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_unset(&mut self) -> Result<Unset<'arena>, ParseError> {
        let unset = self.expect_keyword(T!["unset"])?;
        let result = self.parse_comma_separated_sequence(T!["("], T![")"], |p| p.parse_expression())?;
        let terminator = self.parse_terminator()?;

        Ok(Unset {
            unset,
            left_parenthesis: result.open,
            values: result.sequence,
            right_parenthesis: result.close,
            terminator,
        })
    }
}
