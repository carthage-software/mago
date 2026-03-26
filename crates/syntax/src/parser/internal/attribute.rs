use crate::T;
use crate::ast::ast::Attribute;
use crate::ast::ast::AttributeList;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_attribute_list_sequence(
        &mut self,
    ) -> Result<Sequence<'arena, AttributeList<'arena>>, ParseError> {
        let mut inner = self.new_vec();
        while let Some(T!["#["]) = self.stream.peek_kind(0)? {
            inner.push(self.parse_attribute_list()?);
        }

        Ok(Sequence::new(inner))
    }

    pub(crate) fn parse_attribute_list(&mut self) -> Result<AttributeList<'arena>, ParseError> {
        let result = self.parse_comma_separated_sequence(T!["#["], T!["]"], |p| p.parse_attribute())?;

        Ok(AttributeList { hash_left_bracket: result.open, attributes: result.sequence, right_bracket: result.close })
    }

    pub(crate) fn parse_attribute(&mut self) -> Result<Attribute<'arena>, ParseError> {
        Ok(Attribute { name: self.parse_identifier()?, argument_list: self.parse_optional_argument_list()? })
    }
}
