use crate::cst::identifier::Identifier;
use crate::cst::r#type::GlobalWildcardSelector;
use crate::cst::r#type::GlobalWildcardType;
use crate::cst::r#type::Type;
use crate::cst::r#type::WildcardKind;
use crate::cst::r#type::WildcardType;
use crate::error::ParseError;
use crate::parser::PHPDocParser;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_wildcard_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let token = self.stream.consume()?;
        let file_id = self.file_id();
        let asterisk_span = token.span_for(file_id);

        if self.is_at_member_identifier() {
            let identifier = Identifier::from_token(self.eat_member_identifier()?, file_id);

            Ok(Type::GlobalWildcardReference(GlobalWildcardType {
                selector: GlobalWildcardSelector::EndsWith(asterisk_span, identifier),
            }))
        } else {
            Ok(Type::Wildcard(WildcardType { span: asterisk_span, kind: WildcardKind::Asterisk }))
        }
    }
}
