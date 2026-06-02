use crate::cst::keyword::Keyword;
use crate::error::ParseError;
use crate::parser::PHPDocParser;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_keyword(&mut self) -> Result<Keyword<'arena>, ParseError> {
        let token = self.stream.consume()?;

        Ok(Keyword { span: token.span_for(self.file_id()), value: token.value })
    }
}
