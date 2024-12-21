use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

const STR_STARTS_WITH: &str = "str_starts_with";
const STRPOS: &str = "strpos";

#[derive(Clone, Debug)]
pub struct StrStartsWithRule;

impl Rule for StrStartsWithRule {
    fn get_name(&self) -> &'static str {
        "str-starts-with"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Warning)
    }
}

impl<'a> Walker<LintContext<'a>> for StrStartsWithRule {
    fn walk_in_binary(&self, binary: &Binary, context: &mut LintContext<'a>) {
        let equal = match binary.operator {
            BinaryOperator::Identical(_) | BinaryOperator::Equal(_) => true,
            BinaryOperator::AngledNotEqual(_) | BinaryOperator::NotEqual(_) | BinaryOperator::NotIdentical(_) => false,
            _ => {
                return;
            }
        };

        // if one side is `0` and the other is a `strpos($a, $b)` call, we can suggest using `str_starts_with($a, $b)`
        let (left, call) = match (binary.lhs.as_ref(), binary.rhs.as_ref()) {
            (
                Expression::Literal(Literal::Integer(LiteralInteger { value: Some(0), .. })),
                Expression::Call(Call::Function(call @ FunctionCall { arguments, .. })),
            ) if arguments.arguments.len() == 2 => (false, call),
            (
                Expression::Call(Call::Function(call @ FunctionCall { arguments, .. })),
                Expression::Literal(Literal::Integer(LiteralInteger { value: Some(0), .. })),
            ) if arguments.arguments.len() == 2 => (true, call),
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
            "Consider replacing `strpos` with `str_starts_with` for improved readability and intent clarity.",
        )
        .with_annotation(Annotation::secondary(binary.span()).with_message("This expression can be simplified."))
        .with_help("`strpos($a, $b) === 0` can be simplified to `str_starts_with($a, $b)`.")
        .with_note("Using `str_starts_with` makes the code easier to understand and more expressive.");

        context.report_with_fix(issue, |plan| {
            // we can't guarantee that the replacement is safe, since `strpos` can be re-defined in
            // the current namespace, so we'll mark it as potentially unsafe.
            let safety = SafetyClassification::PotentiallyUnsafe;

            if !equal {
                plan.insert(binary.span().start_position().offset, "!", safety);
            }

            let function_span = function_identifier.span();

            plan.replace(function_span.to_range(), STR_STARTS_WITH.to_string(), safety);

            if left {
                // delete the `=== 0` part
                plan.delete(binary.operator.span().join(binary.rhs.span()).to_range(), safety);
            } else {
                // delete the `0 ===` part
                plan.delete(binary.lhs.span().join(binary.operator.span()).to_range(), safety);
            }
        });
    }
}
