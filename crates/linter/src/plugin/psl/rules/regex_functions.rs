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
pub struct RegexFunctionsRule;

impl Rule for RegexFunctionsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Regex Functions", Level::Warning)
            .with_description(indoc! {"
                This rule enforces the usage of Psl regex functions over their PHP counterparts.

                Psl regex functions are preferred because they are type-safe and provide more consistent behavior.
            "})
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\Regex\\matches` instead of `preg_match`.",
                indoc! {r#"
                    <?php

                    $result = Psl\Regex\matches('Hello, World!', '/\w+/');
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\Regex\\replace` instead of `preg_replace`.",
                indoc! {r#"
                    <?php

                    $result = Psl\Regex\replace('Hello, World!', '/\w+/', 'Goodbye');
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `preg_match`.",
                indoc! {r#"
                    <?php

                    $result = preg_match('/\w+/', 'Hello, World!');
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `preg_replace`.",
                indoc! {r#"
                    <?php

                    $result = preg_replace('/\w+/', 'Goodbye', 'Hello, World!');
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionCall(function_call) = node else { return LintDirective::default() };
        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return LintDirective::default();
        };

        let function_name = context.resolve_function_name(identifier).to_lowercase();

        let replacements = match REGEX_FUNCTION_REPLACEMENTS.get(function_name.as_str()) {
            Some(replacements) => replacements,
            None => return LintDirective::default(),
        };

        context.report(
            Issue::new(
                context.level(),
                "Use the Psl regex function instead of the PHP counterpart.",
            )
            .with_annotation(
                Annotation::primary(identifier.span()).with_message("This is a PHP regex function."),
            )
            .with_note(
                "Psl regex functions are preferred because they are type-safe and provide more consistent behavior.",
            )
            .with_help(format!(
                "Use `{}` instead.",
                format_replacements(replacements)
            )),
        );

        LintDirective::default()
    }
}

static REGEX_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("preg_match", vec!["Psl\\Regex\\matches"]),
        ("preg_match_all", vec!["Psl\\Regex\\matches"]),
        ("preg_replace", vec!["Psl\\Regex\\replace"]),
        ("preg_replace_callback", vec!["Psl\\Regex\\replace_with"]),
        ("preg_replace_callback_array", vec!["Psl\\Regex\\replace_with"]),
        ("preg_split", vec!["Psl\\Regex\\split"]),
        ("preg_grep", vec!["Psl\\Regex\\every_match"]),
        ("preg_filter", vec!["Psl\\Regex\\every_match"]),
        ("preg_quote", vec!["Psl\\Regex\\quote"]),
    ])
});
