use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

const STR_CONTAINS: &str = "str_contains";
const STRPOS: &str = "strpos";

#[derive(Clone, Debug)]
pub struct StrContainsRule;

impl Rule for StrContainsRule {
    fn get_name(&self) -> &'static str {
        "str-contains"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Warning)
    }
}

impl<'a> Walker<LintContext<'a>> for StrContainsRule {
    fn walk_in_binary(&self, binary: &Binary, context: &mut LintContext<'a>) {
        // Detect `strpos($a, $b) !== false`
        if !matches!(
            binary.operator,
            BinaryOperator::NotIdentical(_) | BinaryOperator::NotEqual(_) | BinaryOperator::AngledNotEqual(_)
        ) {
            return;
        }

        let (left, call) = match (binary.lhs.as_ref(), binary.rhs.as_ref()) {
            (
                Expression::Call(Call::Function(call @ FunctionCall { arguments, .. })),
                Expression::Literal(Literal::False(_)),
            ) if arguments.arguments.len() == 2 => (true, call),
            (
                Expression::Literal(Literal::False(_)),
                Expression::Call(Call::Function(call @ FunctionCall { arguments, .. })),
            ) if arguments.arguments.len() == 2 => (false, call),
            _ => {
                return;
            }
        };

        let Expression::Identifier(function_identifier) = call.function.as_ref() else {
            return;
        };

        let function_name = context.resolve_function_name(function_identifier);
        if function_name != STRPOS {
            return;
        }

        let issue = Issue::new(
            context.level(),
            "Consider replacing `strpos` with `str_contains` for improved readability and intent clarity.",
        )
        .with_annotation(Annotation::primary(binary.span()).with_message("This comparison can be simplified."))
        .with_help("`strpos($a, $b) !== false` can be simplified to `str_contains($a, $b)`.")
        .with_note("Using `str_contains` makes the code easier to understand and more expressive.");

        context.report_with_fix(issue, |plan| {
            // Mark the fix as potentially unsafe due to possible redefinition of `strpos` in the namespace.
            let safety = SafetyClassification::PotentiallyUnsafe;

            let function_span = function_identifier.span();

            // Replace `strpos` with `str_contains`
            plan.replace(function_span.to_range(), STR_CONTAINS.to_string(), safety);

            // Remove `!== false` part
            if left {
                plan.delete(binary.operator.span().join(binary.rhs.span()).to_range(), safety);
            } else {
                plan.delete(binary.lhs.span().join(binary.operator.span()).to_range(), safety);
            }
        });
    }
}
