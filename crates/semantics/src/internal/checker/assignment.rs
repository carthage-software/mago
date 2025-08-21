use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_assignment(assignment: &Assignment, context: &mut Context<'_, '_, '_>) {
    if context.version.is_supported(Feature::NullCoalesceAssign) {
        return;
    }

    let AssignmentOperator::Coalesce(operator) = assignment.operator else {
        return;
    };

    context.report(
        Issue::error("The `??=` (null coalesce assignment) operator is only available in PHP 7.4 and later.")
            .with_annotation(
                Annotation::primary(operator.span()).with_message("Null coalesce assignment operator `??=` used here."),
            )
            .with_note("Use a manual check-and-assignment approach if you need compatibility with older PHP versions.")
            .with_help("Replace `$var ??= <default>` with `$var = $var ?? <default>`."),
    );
}
