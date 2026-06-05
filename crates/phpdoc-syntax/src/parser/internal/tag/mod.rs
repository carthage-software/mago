use mago_span::HasSpan;

use crate::cst::identifier::Identifier;
use crate::cst::tag::InvalidTagValue;
use crate::cst::tag::Tag;
use crate::cst::tag::TagValue;
use crate::cst::tag::TagVendor;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::parser::internal::tag::vendor::remainder_after_vendor;
use crate::token::TokenKind;

pub(crate) mod assert;
pub(crate) mod inheritance;
pub(crate) mod meta;
pub(crate) mod method;
pub(crate) mod param;
pub(crate) mod property;
pub(crate) mod template;
pub(crate) mod type_alias;
pub(crate) mod value;
pub(crate) mod vendor;
pub(crate) mod where_clause;

#[inline]
pub(crate) fn is_inherit_doc_name(name: &[u8]) -> bool {
    name.eq_ignore_ascii_case(b"inheritdoc") || name.eq_ignore_ascii_case(b"inheritdocs")
}

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_tag(&mut self) -> Result<Tag<'arena>, ParseError> {
        let file_id = self.file_id();
        let token = self.stream.eat(TokenKind::Tag)?;

        let raw_name = token.value.strip_prefix(b"@").unwrap_or(token.value);
        let name = Identifier { span: token.span_for(file_id), value: raw_name };
        let vendor = TagVendor::from_name(token.value);
        let remainder = remainder_after_vendor(raw_name, vendor);

        let value = match self.dispatch_tag_value(remainder) {
            Ok(value) => value,
            Err(error) => {
                self.record_error(error);
                let text = self.parse_text_or_empty()?;

                TagValue::Invalid(InvalidTagValue { value: text })
            }
        };

        let tag = Tag { name, vendor, value };
        if matches!(tag.value, TagValue::InheritDoc(_)) {
            self.record_inherit_doc(tag.span());
        }

        Ok(tag)
    }

    #[inline]
    fn dispatch_tag_value(&mut self, remainder: &[u8]) -> Result<TagValue<'arena>, ParseError> {
        if is_inherit_doc_name(remainder) {
            return self.parse_inherit_doc_tag_value();
        }

        let mut buffer = [0u8; 48];
        let name = if remainder.len() <= buffer.len() {
            buffer[..remainder.len()].copy_from_slice(remainder);
            buffer[..remainder.len()].make_ascii_lowercase();
            &buffer[..remainder.len()]
        } else {
            remainder
        };

        match name {
            b"param" => self.parse_param_tag_value(),
            b"param-out" => self.parse_param_out_tag_value(),
            b"param-closure-this" => self.parse_param_closure_this_tag_value(),
            b"param-immediately-invoked-callable" => self.parse_param_immediately_invoked_callable_tag_value(),
            b"param-later-invoked-callable" => self.parse_param_later_invoked_callable_tag_value(),
            b"pure-unless-callable-is-impure" => self.parse_pure_unless_callable_is_impure_tag_value(),
            b"return" | b"real-return" => self.parse_return_tag_value(),
            b"var" => self.parse_var_tag_value(),
            b"throws" => self.parse_throws_tag_value(),
            b"mixin" => self.parse_mixin_tag_value(),
            b"self-out" | b"this-out" => self.parse_self_out_tag_value(),
            b"property" | b"property-read" | b"property-write" => self.parse_property_tag_value(),
            b"template" | b"template-covariant" | b"template-contravariant" => self.parse_template_tag(),
            b"extends" | b"inherits" | b"template-extends" => self.parse_extends_tag_value(),
            b"implements" | b"template-implements" => self.parse_implements_tag_value(),
            b"use" | b"uses" | b"template-use" => self.parse_uses_tag_value(),
            b"require-extends" => self.parse_require_extends_tag_value(),
            b"require-implements" => self.parse_require_implements_tag_value(),
            b"sealed" => self.parse_sealed_tag_value(),
            b"inheritors" => self.parse_inheritors_tag_value(),
            b"method" => self.parse_method_tag_value(),
            b"assert" | b"assert-if-true" | b"assert-if-false" => self.parse_assert_tag_value(),
            b"where" => self.parse_where_tag_value(),
            b"type" => self.parse_type_alias_tag_value(),
            b"import-type" => self.parse_type_alias_import_tag_value(),
            b"deprecated" => self.parse_deprecated_tag_value(),
            _ => self.parse_generic_tag_value(),
        }
    }
}
