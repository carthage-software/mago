use mago_ast::ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Copy, Debug)]
pub struct ImplicitlyNullableParameterRule;

impl Rule for ImplicitlyNullableParameterRule {
    fn get_name(&self) -> &'static str {
        "implicitly-nullable-parameter"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Warning)
    }
}

impl<'a> Walker<LintContext<'a>> for ImplicitlyNullableParameterRule {
    fn walk_function_like_parameter(
        &self,
        function_like_parameter: &FunctionLikeParameter,
        context: &mut LintContext<'a>,
    ) {
        let Some(hint) = function_like_parameter.hint.as_ref() else {
            return;
        };

        if hint.contains_null() {
            return;
        }

        let Some(default_value) = function_like_parameter.default_value.as_ref() else {
            return;
        };

        let Expression::Literal(Literal::Null(_)) = default_value.value else {
            return;
        };

        let parameter_name = context.lookup(&function_like_parameter.variable.name);
        let current_hint = context.get_readable_hint(hint);
        let replacement_hint = match hint {
            Hint::Union(_) => format!("null|{}", current_hint),
            Hint::Intersection(_) => format!("null|({})", current_hint),
            Hint::Parenthesized(_) => format!("null|{}", current_hint),
            _ => format!("?{}", current_hint),
        };

        let issue = Issue::new(
            context.level(),
            format!("Parameter `{}` is implicitly nullable and relies on a deprecated feature.", parameter_name),
        )
        .with_annotation(
            Annotation::primary(function_like_parameter.span())
                .with_message(format!("Parameter `{}` is declared here.", parameter_name)),
        )
        .with_help(format!(
            "Consider using an explicit nullable type hint ( `{}` ) or replacing the default value.",
            replacement_hint
        ))
        .with_note("Updating this will future-proof your code and align it with PHP 8.4 standards.");

        context.report_with_fix(issue, |plan| {
            plan.replace(hint.span().to_range(), replacement_hint, SafetyClassification::Safe);
        });
    }
}
