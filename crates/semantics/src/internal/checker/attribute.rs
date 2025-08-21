use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_attribute_list(attribute_list: &AttributeList, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::Attributes) {
        context.report(
            Issue::error("Attributes are only available in PHP 8.0 and above.")
                .with_annotation(Annotation::primary(attribute_list.span()).with_message("Attribute list used here."))
                .with_help("Upgrade to PHP 8.0 or above to use attributes."),
        );
    }

    for attr in attribute_list.attributes.iter() {
        let name = attr.name.value();

        if let Some(list) = &attr.argument_list {
            for argument in list.arguments.iter() {
                let (ellipsis, value) = match &argument {
                    Argument::Positional(positional_argument) => {
                        (positional_argument.ellipsis.as_ref(), &positional_argument.value)
                    }
                    Argument::Named(named_argument) => (None, &named_argument.value),
                };

                if let Some(ellipsis) = ellipsis {
                    context.report(
                        Issue::error("Cannot use argument unpacking in attribute arguments.")
                            .with_annotation(
                                Annotation::primary(ellipsis.span()).with_message("Argument unpacking used here."),
                            )
                            .with_annotation(
                                Annotation::secondary(attr.name.span())
                                    .with_message(format!("Attribute `{name}` defined here.")),
                            )
                            .with_note("Unpacking arguments is not allowed in attribute arguments."),
                    );
                }

                if !value.is_constant(&context.version, true) {
                    context.report(
                        Issue::error(format!("Attribute `{name}` argument contains a non-constant expression."))
                            .with_annotations([
                                Annotation::primary(value.span()).with_message("Non-constant expression used here."),
                                Annotation::secondary(attr.name.span())
                                    .with_message(format!("Attribute `{name}` defined here.")),
                            ])
                            .with_note("Attribute arguments must be constant expressions."),
                    );
                }
            }
        }
    }
}
