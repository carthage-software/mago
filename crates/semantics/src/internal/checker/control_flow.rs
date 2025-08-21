use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_switch(switch: &Switch, context: &mut Context<'_, '_, '_>) {
    let mut last_default: Option<Span> = None;

    for case in switch.body.cases() {
        let SwitchCase::Default(default_case) = &case else {
            continue;
        };

        let Some(previous) = last_default else {
            last_default = Some(default_case.default.span);

            continue;
        };

        context.report(
            Issue::error("A switch statement can only have one default case.")
                .with_annotation(
                    Annotation::primary(default_case.span()).with_message("This is a duplicate default case."),
                )
                .with_annotation(
                    Annotation::secondary(previous).with_message("The first default case is defined here."),
                )
                .with_annotation(
                    Annotation::secondary(switch.span())
                        .with_message("Switch statement containing the duplicate cases."),
                )
                .with_help(
                    "Remove this duplicate default case to ensure the switch statement is valid and unambiguous.",
                ),
        );
    }
}

#[inline]
pub fn check_match(r#match: &Match, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::MatchExpression) {
        context.report(
            Issue::error("Match expressions are only available in PHP 8.0 and above.")
                .with_annotation(Annotation::primary(r#match.span()).with_message("Match expression defined here."))
                .with_help("Upgrade to PHP 8.0 or above to use match expressions."),
        );
    }

    let mut last_default: Option<Span> = None;
    for arm in r#match.arms.iter() {
        let MatchArm::Default(default_arm) = &arm else {
            continue;
        };

        let Some(previous) = last_default else {
            last_default = Some(default_arm.default.span);

            continue;
        };

        context.report(
            Issue::error("A match expression can only have one default arm.")
                .with_annotation(
                    Annotation::primary(default_arm.span()).with_message("This is a duplicate default arm."),
                )
                .with_annotation(Annotation::secondary(previous).with_message("The first default arm is defined here."))
                .with_annotation(Annotation::secondary(r#match.span()).with_message("Match expression defined here."))
                .with_help("Remove this duplicate default arm to ensure the match expression is valid."),
        );
    }
}
