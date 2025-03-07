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
pub struct SleepFunctionsRule;

impl Rule for SleepFunctionsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Sleep Functions", Level::Warning)
            .with_description(indoc! {"
                This rule enforces the usage of Psl sleep functions over their PHP counterparts.

                Psl sleep functions are preferred because they are type-safe, provide more consistent behavior,
                and allow other tasks within the event loop to continue executing while the current Fiber pauses.
            "})
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\Async\\sleep` instead of `sleep`.",
                indoc! {r#"
                    <?php

                    use Psl\Async;
                    use Psl\DateTime;

                    Async\sleep(DateTime\Duration::seconds(1));
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\Async\\sleep` instead of `usleep`.",
                indoc! {r#"
                    <?php

                    use Psl\Async;
                    use Psl\DateTime;

                    Async\sleep(DateTime\Duration::milliseconds(1500));
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `sleep`.",
                indoc! {r#"
                    <?php

                    sleep(1);
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `usleep`.",
                indoc! {r#"
                    <?php

                    usleep(1500);
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionCall(function_call) = node else { return LintDirective::default() };
        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return LintDirective::default();
        };

        let function_name = context.resolve_function_name(identifier).to_lowercase();

        let Some(replacements) = (*SLEEP_FUNCTION_REPLACEMENTS).get(function_name.as_str()) else {
            return LintDirective::default();
        };

        context.report(
            Issue::new(
                context.level(),
                "Use the Psl sleep function instead of the PHP counterpart.",
            )
            .with_annotation(Annotation::primary(identifier.span()).with_message("This is a PHP sleep function."))
            .with_note("Psl sleep functions are preferred because they are type-safe, provide more consistent behavior, and allow other tasks within the event loop to continue executing while the current Fiber pauses.")
            .with_help(format!("Use `{}` instead.", format_replacements(replacements))),
        );

        LintDirective::default()
    }
}

static SLEEP_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("sleep", vec!["Psl\\Async\\sleep"]),
        ("usleep", vec!["Psl\\Async\\sleep"]),
        ("time_sleep_until", vec!["Psl\\Async\\sleep"]),
        ("time_nanosleep", vec!["Psl\\Async\\sleep"]),
    ])
});
