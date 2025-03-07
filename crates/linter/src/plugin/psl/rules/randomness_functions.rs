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
pub struct RandomnessFunctionsRule;

impl Rule for RandomnessFunctionsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Randomness Functions", Level::Warning)
            .with_description(indoc! {"
                This rule enforces the usage of Psl randomness functions over their PHP counterparts.

                Psl randomness functions are preferred because they are type-safe and provide more consistent behavior.
            "})
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\SecureRandom\\int` instead of `random_int`.",
                indoc! {r#"
                    <?php

                    $randomInt = Psl\SecureRandom\int(0, 10);
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\PseudoRandom\\int` instead of `rand`.",
                indoc! {r#"
                    <?php

                    $randomInt = Psl\PseudoRandom\int(0, 10);
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `random_int`.",
                indoc! {r#"
                    <?php

                    $randomInt = random_int(0, 10);
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `rand`.",
                indoc! {r#"
                    <?php

                    $randomInt = rand(0, 10);
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionCall(function_call) = node else { return LintDirective::default() };
        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return LintDirective::default();
        };

        let function_name = context.resolve_function_name(identifier).to_lowercase();

        let replacements = match RANDOM_FUNCTION_REPLACEMENTS.get(function_name.as_str()) {
            Some(replacements) => replacements,
            None => return LintDirective::default(),
        };

        context.report(
            Issue::new(
                context.level(),
                "Use the Psl randomness function instead of the PHP counterpart.",
            )
            .with_annotation(
                Annotation::primary(identifier.span()).with_message("This is a PHP randomness function."),
            )
            .with_note(
                "Psl randomness functions are preferred because they are type-safe and provide more consistent behavior.",
            )
            .with_help(format!(
                "Use `{}` instead.",
                format_replacements(replacements),
            )),
        );

        LintDirective::default()
    }
}

static RANDOM_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("rand", vec!["Psl\\PseudoRandom\\int", "Psl\\PseudoRandom\\float"]),
        ("mt_rand", vec!["Psl\\PseudoRandom\\int", "Psl\\PseudoRandom\\float"]),
        ("random_int", vec!["Psl\\SecureRandom\\int", "Psl\\SecureRandom\\float"]),
        ("random_bytes", vec!["Psl\\SecureRandom\\bytes", "Psl\\SecureRandom\\string"]),
    ])
});
