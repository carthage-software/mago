use mago_allocator::Arena;

use crate::cst::identifier::Identifier;
use crate::cst::tag::InvalidTagValue;
use crate::cst::tag::Tag;
use crate::cst::tag::TagValue;
use crate::cst::tag::TagVendor;
use crate::cst::tag::TemplateTagValueVariance;
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

const NORMALIZED_NAME_CAPACITY: usize = 48;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
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

        let tag = Tag { at: token.span_for(file_id).subspan(0, 1), name, vendor, value };

        Ok(tag)
    }

    #[inline]
    fn dispatch_tag_value(&mut self, remainder: &[u8]) -> Result<TagValue<'arena>, ParseError> {
        let mut buffer = [0u8; NORMALIZED_NAME_CAPACITY];
        let mut length = 0;
        for byte in remainder {
            if *byte == b'-' {
                continue;
            }

            if length == buffer.len() {
                return self.parse_generic_tag_value();
            }

            buffer[length] = byte.to_ascii_lowercase();
            length += 1;
        }

        let name = &buffer[..length];

        match name {
            b"param" => self.parse_param_tag_value(),
            b"paramout" => self.parse_param_out_tag_value(),
            b"paramclosurethis" => self.parse_param_closure_this_tag_value(),
            b"paramimmediatelyinvokedcallable" => self.parse_param_immediately_invoked_callable_tag_value(),
            b"paramlaterinvokedcallable" => self.parse_param_later_invoked_callable_tag_value(),
            b"pureunlesscallableisimpure" => self.parse_pure_unless_callable_is_impure_tag_value(),
            b"return" => Ok(TagValue::Return(self.parse_return_tag_value()?)),
            b"realreturn" => Ok(TagValue::RealReturn(self.parse_return_tag_value()?)),
            b"var" => self.parse_var_tag_value(),
            b"throws" => self.parse_throws_tag_value(),
            b"mixin" => self.parse_mixin_tag_value(),
            b"selfout" | b"thisout" => self.parse_self_out_tag_value(),
            b"property" => Ok(TagValue::Property(self.parse_property_tag_value()?)),
            b"propertyread" => Ok(TagValue::PropertyRead(self.parse_property_tag_value()?)),
            b"propertywrite" => Ok(TagValue::PropertyWrite(self.parse_property_tag_value()?)),
            b"template" | b"templateinvariant" => self.parse_template_tag(TemplateTagValueVariance::Invariant),
            b"templatecovariant" => self.parse_template_tag(TemplateTagValueVariance::Covariant),
            b"templatecontravariant" => self.parse_template_tag(TemplateTagValueVariance::Contravariant),
            b"extends" | b"templateextends" => self.parse_extends_tag_value(),
            b"implements" | b"templateimplements" => self.parse_implements_tag_value(),
            b"use" | b"templateuse" => self.parse_use_tag_value(),
            b"requireextends" => self.parse_require_extends_tag_value(),
            b"requireimplements" => self.parse_require_implements_tag_value(),
            b"sealed" => self.parse_sealed_tag_value(),
            b"inheritors" => self.parse_inheritors_tag_value(),
            b"method" => self.parse_method_tag_value(),
            b"assert" => Ok(TagValue::Assert(self.parse_assert_tag_value()?)),
            b"assertiftrue" => Ok(TagValue::AssertIfTrue(self.parse_assert_tag_value()?)),
            b"assertiffalse" => Ok(TagValue::AssertIfFalse(self.parse_assert_tag_value()?)),
            b"where" => self.parse_where_tag_value(),
            b"type" => self.parse_type_alias_tag_value(),
            b"importtype" => self.parse_type_alias_import_tag_value(),
            b"deprecated" => self.parse_deprecated_tag_value(),
            b"notdeprecated" => self.parse_not_deprecated_tag_value(),
            b"inheritdoc" | b"inheritdocs" => self.parse_inherit_doc_tag_value(),
            b"final" => self.parse_final_tag_value(),
            b"internal" => self.parse_internal_tag_value(),
            b"api" => self.parse_api_tag_value(),
            b"experimental" => self.parse_experimental_tag_value(),
            b"pure" => self.parse_pure_tag_value(),
            b"impure" => self.parse_impure_tag_value(),
            b"readonly" => self.parse_readonly_tag_value(),
            b"mustuse" => self.parse_must_use_tag_value(),
            b"nonamedarguments" => self.parse_no_named_arguments_tag_value(),
            b"enuminterface" => self.parse_enum_interface_tag_value(),
            b"consistentconstructor" => self.parse_consistent_constructor_tag_value(),
            b"consistenttemplates" => self.parse_consistent_templates_tag_value(),
            b"sealproperties" => self.parse_seal_properties_tag_value(),
            b"nosealproperties" => self.parse_no_seal_properties_tag_value(),
            b"sealmethods" => self.parse_seal_methods_tag_value(),
            b"nosealmethods" => self.parse_no_seal_methods_tag_value(),
            b"mutationfree" => self.parse_mutation_free_tag_value(),
            b"externalmutationfree" => self.parse_external_mutation_free_tag_value(),
            b"suspendsfiber" => self.parse_suspends_fiber_tag_value(),
            b"ignorenullablereturn" => self.parse_ignore_nullable_return_tag_value(),
            b"ignorefalsablereturn" => self.parse_ignore_falsable_return_tag_value(),
            b"trace" => self.parse_trace_tag_value(),
            _ => self.parse_generic_tag_value(),
        }
    }
}
