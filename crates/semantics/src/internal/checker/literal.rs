use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_literal(literal: &Literal, context: &mut Context<'_, '_, '_>) {
    if context.version.is_supported(Feature::NumericLiteralSeparator) {
        return;
    }

    let value = match literal {
        Literal::Integer(literal_integer) => &literal_integer.raw,
        Literal::Float(literal_float) => &literal_float.raw,
        _ => return,
    };

    if !value.contains('_') {
        return;
    }

    context.report(
        Issue::error("Numeric literal separators are only available in PHP 7.4 and later.")
            .with_annotation(Annotation::primary(literal.span()).with_message("Numeric literal used here."))
            .with_help("Remove the underscore separators to make the code compatible with PHP 7.3 and earlier versions, or upgrade to PHP 7.4 or later."),
    );
}
