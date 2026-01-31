use crate::T;
use crate::ast::ast::Attribute;
use crate::ast::ast::AttributeList;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_attribute_list_sequence(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Sequence<'arena, AttributeList<'arena>>, ParseError> {
        let mut inner = self.new_vec();
        while let Some(T!["#["]) = stream.lookahead(0)?.map(|t| t.kind) {
            inner.push(self.parse_attribute_list(stream)?);
        }

        Ok(Sequence::new(inner))
    }

    pub(crate) fn parse_attribute_list(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<AttributeList<'arena>, ParseError> {
        let result = self.parse_comma_separated_sequence(stream, T!["#["], T!["]"], |p, s| p.parse_attribute(s))?;

        Ok(AttributeList { hash_left_bracket: result.open, attributes: result.sequence, right_bracket: result.close })
    }

    pub(crate) fn parse_attribute(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Attribute<'arena>, ParseError> {
        Ok(Attribute {
            name: self.parse_identifier(stream)?,
            argument_list: self.parse_optional_argument_list(stream)?,
        })
    }
}
