use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::PartialApplication;
use mago_syntax::cst::PartialArgument;

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

#[inline]
pub fn report_disallowed_partial_argument(
    argument: &PartialArgument,
    position: &str,
    owner_span: Span,
    owner_message: String,
    context: &mut Context<'_, '_, '_>,
) {
    let (span, what) = match argument {
        PartialArgument::Placeholder(placeholder) => (placeholder.span, "Partial function application"),
        PartialArgument::NamedPlaceholder(placeholder) => (placeholder.span(), "Partial function application"),
        PartialArgument::VariadicPlaceholder(placeholder) => (placeholder.span, "First-class callable syntax"),
        PartialArgument::Positional(_) | PartialArgument::Named(_) => return,
    };

    context.report(
        Issue::error(format!("{what} cannot be used in {position}."))
            .with_annotation(Annotation::primary(span).with_message(format!("{what} used here.")))
            .with_annotation(Annotation::secondary(owner_span).with_message(owner_message))
            .with_note(
                "Placeholders and first-class callable syntax are only allowed in direct function and method calls.",
            )
            .with_help("Pass concrete arguments instead."),
    );
}
