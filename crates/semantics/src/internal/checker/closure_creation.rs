use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_closure_creation(closure_creation: &ClosureCreation, context: &mut Context<'_, '_, '_>) {
    if context.version.is_supported(Feature::ClosureCreation) {
        return;
    }

    context.report(
        Issue::error("The closure creation syntax is only available in PHP 8.1 and above.").with_annotation(
            Annotation::primary(closure_creation.span()).with_message("Closure creation syntax used here."),
        ),
    );
}
