use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_partial_application(partial_application: &PartialApplication, context: &mut Context<'_, '_, '_>) {
    let argument_list = partial_application.get_argument_list();

    if argument_list.is_first_class_callable() {
        if context.version.is_supported(Feature::ClosureCreation) {
            return;
        }

        context.report(
            Issue::error("First-class callable syntax is only available in PHP 8.1 and above.")
                .with_annotation(
                    Annotation::primary(partial_application.span())
                        .with_message("First-class callable syntax used here."),
                )
                .with_help("Upgrade to PHP 8.1 or above to use first-class callables."),
        );
    } else {
        if context.version.is_supported(Feature::PartialFunctionApplication) {
            return;
        }

        context.report(
            Issue::error("Partial function application is only available in PHP 8.6 and above.")
                .with_annotation(
                    Annotation::primary(partial_application.span())
                        .with_message("Partial function application used here."),
                )
                .with_note(
                    "Partial function application uses placeholders like `?` or named placeholders to create partially applied functions.",
                )
                .with_help("Upgrade to PHP 8.6 or above to use partial function application."),
        );
    }
}
