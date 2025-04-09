use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_attribute_list(attribute_list: &AttributeList, context: &mut Context<'_>) {
    if !context.version.is_supported(Feature::Attribute) {
        context.issues.push(
            Issue::error("Attributes are only available in PHP 8.0 and above.")
                .with_annotation(Annotation::primary(attribute_list.span()).with_message("Attribute list used here."))
                .with_help("Upgrade to PHP 8.0 or above to use attributes."),
        );
    }

    for attr in attribute_list.attributes.iter() {
        let name = context.interner.lookup(attr.name.value());
        if let Some(list) = &attr.arguments {
            for argument in list.arguments.iter() {
                let (ellipsis, value) = match &argument {
                    Argument::Positional(positional_argument) => {
                        (positional_argument.ellipsis, &positional_argument.value)
                    }
                    Argument::Named(named_argument) => (named_argument.ellipsis, &named_argument.value),
                };

                if let Some(ellipsis) = ellipsis {
                    context.issues.push(
                        Issue::error("Cannot use argument unpacking in attribute arguments.")
                            .with_annotation(
                                Annotation::primary(ellipsis.span()).with_message("Argument unpacking used here."),
                            )
                            .with_annotation(
                                Annotation::secondary(attr.name.span())
                                    .with_message(format!("Attribute `{}` defined here.", name)),
                            )
                            .with_note("Unpacking arguments is not allowed in attribute arguments."),
                    );
                }

                if !value.is_constant(context.version, true) {
                    context.issues.push(
                        Issue::error(format!("Attribute `{}` argument contains a non-constant expression.", name))
                            .with_annotations([
                                Annotation::primary(value.span()).with_message("Non-constant expression used here."),
                                Annotation::secondary(attr.name.span())
                                    .with_message(format!("Attribute `{}` defined here.", name)),
                            ])
                            .with_note("Attribute arguments must be constant expressions."),
                    );
                }
            }
        }
    }
}
