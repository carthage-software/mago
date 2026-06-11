use mago_allocator::Arena;

use crate::cst::tag::ApiTagValue;
use crate::cst::tag::ConsistentConstructorTagValue;
use crate::cst::tag::ConsistentTemplatesTagValue;
use crate::cst::tag::DeprecatedTagValue;
use crate::cst::tag::EnumInterfaceTagValue;
use crate::cst::tag::ExperimentalTagValue;
use crate::cst::tag::ExternalMutationFreeTagValue;
use crate::cst::tag::FinalTagValue;
use crate::cst::tag::GenericTagValue;
use crate::cst::tag::IgnoreFalsableReturnTagValue;
use crate::cst::tag::IgnoreNullableReturnTagValue;
use crate::cst::tag::ImpureTagValue;
use crate::cst::tag::InheritDocTagValue;
use crate::cst::tag::InternalTagValue;
use crate::cst::tag::MustUseTagValue;
use crate::cst::tag::MutationFreeTagValue;
use crate::cst::tag::NoNamedArgumentsTagValue;
use crate::cst::tag::NoSealMethodsTagValue;
use crate::cst::tag::NoSealPropertiesTagValue;
use crate::cst::tag::NotDeprecatedTagValue;
use crate::cst::tag::PureTagValue;
use crate::cst::tag::PureUnlessCallableIsImpureTagValue;
use crate::cst::tag::ReadonlyTagValue;
use crate::cst::tag::SealMethodsTagValue;
use crate::cst::tag::SealPropertiesTagValue;
use crate::cst::tag::SuspendsFiberTagValue;
use crate::cst::tag::TagValue;
use crate::cst::tag::TraceTagValue;
use crate::error::ParseError;
use crate::parser::PHPDocParser;

macro_rules! marker_tag_value_parsers {
    ($($parser:ident => $variant:ident($value:ident)),+ $(,)?) => {
        $(
            pub(crate) fn $parser(&mut self) -> Result<TagValue<'arena>, ParseError> {
                let description = self.parse_text_or_empty()?;

                Ok(TagValue::$variant($value { description }))
            }
        )+
    };
}

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    marker_tag_value_parsers!(
        parse_deprecated_tag_value => Deprecated(DeprecatedTagValue),
        parse_not_deprecated_tag_value => NotDeprecated(NotDeprecatedTagValue),
        parse_inherit_doc_tag_value => InheritDoc(InheritDocTagValue),
        parse_final_tag_value => Final(FinalTagValue),
        parse_internal_tag_value => Internal(InternalTagValue),
        parse_api_tag_value => Api(ApiTagValue),
        parse_experimental_tag_value => Experimental(ExperimentalTagValue),
        parse_pure_tag_value => Pure(PureTagValue),
        parse_impure_tag_value => Impure(ImpureTagValue),
        parse_readonly_tag_value => Readonly(ReadonlyTagValue),
        parse_must_use_tag_value => MustUse(MustUseTagValue),
        parse_no_named_arguments_tag_value => NoNamedArguments(NoNamedArgumentsTagValue),
        parse_enum_interface_tag_value => EnumInterface(EnumInterfaceTagValue),
        parse_consistent_constructor_tag_value => ConsistentConstructor(ConsistentConstructorTagValue),
        parse_consistent_templates_tag_value => ConsistentTemplates(ConsistentTemplatesTagValue),
        parse_seal_properties_tag_value => SealProperties(SealPropertiesTagValue),
        parse_no_seal_properties_tag_value => NoSealProperties(NoSealPropertiesTagValue),
        parse_seal_methods_tag_value => SealMethods(SealMethodsTagValue),
        parse_no_seal_methods_tag_value => NoSealMethods(NoSealMethodsTagValue),
        parse_mutation_free_tag_value => MutationFree(MutationFreeTagValue),
        parse_external_mutation_free_tag_value => ExternalMutationFree(ExternalMutationFreeTagValue),
        parse_suspends_fiber_tag_value => SuspendsFiber(SuspendsFiberTagValue),
        parse_ignore_nullable_return_tag_value => IgnoreNullableReturn(IgnoreNullableReturnTagValue),
        parse_ignore_falsable_return_tag_value => IgnoreFalsableReturn(IgnoreFalsableReturnTagValue),
    );

    pub(crate) fn parse_trace_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let variable = self.parse_variable()?;
        let description = self.parse_text_or_empty()?;

        Ok(TagValue::Trace(TraceTagValue { variable, description }))
    }

    pub(crate) fn parse_pure_unless_callable_is_impure_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let parameter = self.parse_variable()?;
        let description = self.parse_optional_description(false)?;

        Ok(TagValue::PureUnlessCallableIsImpure(PureUnlessCallableIsImpureTagValue { parameter, description }))
    }

    pub(crate) fn parse_generic_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let value = self.parse_text_or_empty()?;

        Ok(TagValue::Generic(GenericTagValue { value }))
    }
}
