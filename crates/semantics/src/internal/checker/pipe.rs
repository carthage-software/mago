use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_pipe(pipe: &Pipe, context: &mut Context<'_, '_, '_>) {
    if context.version.is_supported(Feature::PipeOperator) {
        return;
    }

    context.report(
        Issue::error(format!(
            "The pipe operator (`|>`) is not available in your configured PHP version ({}).",
            context.version
        ))
        .with_annotation(
            Annotation::primary(pipe.operator.span()).with_message("Pipe operator (`|>`) used here"),
        )
        .with_note("This feature was introduced in PHP 8.5 and allows for a more readable way to chain operations by passing the result of the left-hand expression as the first argument to the right-hand callable.")
        .with_help("To use the pipe operator, please ensure your project targets PHP 8.5 or newer.")
        .with_link("https://wiki.php.net/rfc/pipe-operator-v3"),
    );
}
