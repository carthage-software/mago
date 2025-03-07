use std::sync::LazyLock;

use ahash::HashMap;
use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::plugin::psl::rules::utils::format_replacements;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct MathFunctionsRule;

impl Rule for MathFunctionsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Math Functions", Level::Warning)
            .with_description(indoc! {"
                   This rule enforces the usage of Psl math functions over their PHP counterparts.

                   Psl math functions are preferred because they are type-safe and provide more consistent behavior.
               "})
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\Math\\abs` instead of `abs`.",
                indoc! {r#"
                    <?php

                    $abs = Psl\Math\abs($number);
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\Math\\maxva` instead of `max`.",
                indoc! {r#"
                    <?php

                    $max = Psl\Math\maxva($numbers);
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `abs`.",
                indoc! {r#"
                    <?php

                    $abs = abs($number);
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `max`.",
                indoc! {r#"
                    <?php

                    $max = max(...$numbers);
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionCall(function_call) = node else { return LintDirective::default() };
        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return LintDirective::default();
        };

        let function_name = context.resolve_function_name(identifier).to_lowercase();
        if let Some(replacements) = MATH_FUNCTION_REPLACEMENTS.get(function_name.as_str()) {
            context.report(
                Issue::new(context.level(), "Use the Psl math function instead of the PHP counterpart.")
                    .with_annotation(Annotation::primary(identifier.span()).with_message("This is a PHP math function."))
                    .with_note(
                        "Psl math functions are preferred because they are type-safe and provide more consistent behavior.",
                    )
                    .with_help(format!("Use `{}` instead.", format_replacements(replacements))),
            );
        }

        LintDirective::default()
    }
}

static MATH_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("abs", vec!["Psl\\Math\\abs"]),
        ("acos", vec!["Psl\\Math\\acos"]),
        ("acos", vec!["Psl\\Math\\acos"]),
        ("asin", vec!["Psl\\Math\\asin"]),
        ("atan", vec!["Psl\\Math\\atan2"]),
        ("base_convert", vec!["Psl\\Math\\base_convert"]),
        ("ceil", vec!["Psl\\Math\\ceil"]),
        ("cos", vec!["Psl\\Math\\cos"]),
        ("intdiv", vec!["Psl\\Math\\div"]),
        ("exp", vec!["Psl\\Math\\exp"]),
        ("floor", vec!["Psl\\Math\\floor"]),
        ("hexdec", vec!["Psl\\Math\\from_base"]),
        ("bindec", vec!["Psl\\Math\\from_base"]),
        ("decbin", vec!["Psl\\Math\\to_base"]),
        ("dechex", vec!["Psl\\Math\\to_base"]),
        ("decoct", vec!["Psl\\Math\\to_base"]),
        ("log", vec!["Psl\\Math\\log"]),
        ("max", vec!["Psl\\Math\\max", "Psl\\Math\\maxva", "Psl\\Math\\max_by"]),
        ("min", vec!["Psl\\Math\\min", "Psl\\Math\\minva", "Psl\\Math\\min_by"]),
        ("round", vec!["Psl\\Math\\round"]),
        ("sin", vec!["Psl\\Math\\sin"]),
        ("sqrt", vec!["Psl\\Math\\sqrt"]),
        ("tan", vec!["Psl\\Math\\tan"]),
    ])
});
