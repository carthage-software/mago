use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_argument_list<'ast, 'arena>(
    argument_list: &'ast ArgumentList<'arena>,
    context: &mut Context<'_, 'ast, 'arena>,
) {
    let mut last_named_argument: Option<Span> = None;
    let mut last_unpacking: Option<Span> = None;

    for argument in argument_list.arguments.iter() {
        match &argument {
            Argument::Positional(positional_argument) => {
                if let Some(ellipsis) = positional_argument.ellipsis {
                    if let Some(last_named_argument) = last_named_argument {
                        context.report(
                            Issue::error("Cannot use argument unpacking after a named argument.")
                                .with_annotation(
                                    Annotation::primary(ellipsis.span()).with_message("Unpacking argument here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(last_named_argument).with_message("Named argument here."),
                                )
                                .with_note("Unpacking arguments must come before named arguments."),
                        );
                    }

                    last_unpacking = Some(ellipsis.span());
                } else {
                    if let Some(named_argument) = last_named_argument {
                        context.report(
                            Issue::error("Cannot use positional argument after a named argument.")
                                .with_annotation(
                                    Annotation::primary(positional_argument.span())
                                        .with_message("Positional argument defined here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(named_argument).with_message("Named argument here."),
                                )
                                .with_note("Positional arguments must come before named arguments."),
                        );
                    }

                    if let Some(unpacking) = last_unpacking {
                        context.report(
                            Issue::error("Cannot use positional argument after argument unpacking.")
                                .with_annotation(
                                    Annotation::primary(positional_argument.span())
                                        .with_message("Positional argument defined here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(unpacking).with_message("Argument unpacking here."),
                                )
                                .with_note("Positional arguments must come before unpacking arguments."),
                        );
                    }
                }
            }
            Argument::Named(named_argument) => {
                if !context.version.is_supported(Feature::NamedArguments) {
                    context.report(
                        Issue::error("Named arguments are only available in PHP 8.0 and above.").with_annotation(
                            Annotation::primary(named_argument.span()).with_message("Named argument used here."),
                        ),
                    );
                }

                last_named_argument = Some(named_argument.span());
            }
        }
    }

    if !context.version.is_supported(Feature::TrailingCommaInFunctionCalls)
        && let Some(last_comma) = argument_list.arguments.get_trailing_token()
    {
        context.report(
                Issue::error("Trailing comma in function calls is only available in PHP 7.3 and later.")
                    .with_annotation(
                        Annotation::primary(last_comma.span).with_message("Trailing comma found here."),
                    )
                    .with_help(
                        "Remove the trailing comma to make the code compatible with PHP 7.2 and earlier versions, or upgrade to PHP 7.3 or later.",
                    )
            );
    }
}
