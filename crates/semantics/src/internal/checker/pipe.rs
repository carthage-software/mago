use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_pipe(pipe: &Pipe, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::PipeOperator) {
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

        return;
    }

    if let Expression::ArrowFunction(arrow_function) = pipe.callable {
        context.report(
            Issue::error("Arrow function on the right side of the pipe operator must be parenthesized.".to_string())
            .with_annotation(
                Annotation::primary(arrow_function.span()).with_message("Unparenthesized arrow function used here."),
            )
            .with_note("When using arrow functions with the pipe operator, the arrow function must be enclosed in parentheses to ensure correct parsing.")
            .with_help("Wrap the arrow function in parentheses, e.g., `$foo |> (fn($x) => $x + 1)`.")
            .with_link("https://externals.io/message/128473#128473"),
        );
    }
}
