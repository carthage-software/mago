use std::sync::LazyLock;

use ahash::HashMap;
use indoc::indoc;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::plugin::psl::rules::utils::format_replacements;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct ArrayFunctionsRule;

impl Rule for ArrayFunctionsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Array Functions", Level::Warning)
            .with_description(indoc! {"
                  This rule enforces the usage of Psl array functions over their PHP counterparts.

                  Psl array functions are preferred because they are type-safe and provide more consistent behavior.
              "})
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\Vec\\filter` instead of `array_filter`.",
                indoc! {r#"
                    <?php

                    $filtered = Psl\Vec\filter($xs, fn($x) => $x > 2);
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\Vec\\map` instead of `array_map`.",
                indoc! {r#"
                    <?php

                    $mapped = Psl\Vec\map($xs, fn($x) => $x * 2);
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `array_filter`.",
                indoc! {r#"
                    <?php

                    $filtered = array_filter($xs, fn($x) => $x > 2);
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `array_map`.",
                indoc! {r#"
                    <?php

                    $mapped = array_map($xs, fn($x) => $x * 2);
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionCall(function_call) = node else { return LintDirective::default() };
        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return LintDirective::default();
        };

        let function_name = context.resolve_function_name(identifier).to_lowercase();
        if let Some(replacements) = (*ARRAY_FUNCTION_REPLACEMENTS).get(function_name.as_str()) {
            context.report(
                Issue::new(
                    context.level(),
                    "Use the Psl array function instead of the PHP counterpart.",
                )
                .with_annotation(
                    Annotation::primary(identifier.span()).with_message("This is a PHP array function."),
                )
                .with_note("Psl array functions are preferred because they are type-safe and provide more consistent behavior.")
                .with_help(format!(
                    "Use `{}` instead.",
                    format_replacements(replacements),
                )),
            );
        }

        LintDirective::default()
    }
}

static ARRAY_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("array_all", vec!["Psl\\Iter\\all"]),
        ("array_any", vec!["Psl\\Iter\\any"]),
        ("array_chunk", vec!["Psl\\Vec\\chunk", "Psl\\Vec\\chunk_with_keys"]),
        ("array_combine", vec!["Psl\\Dict\\associate"]),
        ("array_diff", vec!["Psl\\Dict\\diff"]),
        ("array_diff_key", vec!["Psl\\Dict\\diff_by_key"]),
        ("array_fill", vec!["Psl\\Vec\\fill"]),
        (
            "array_filter",
            vec![
                "Psl\\Vec\\filter",
                "Psl\\Vec\\filter_keys",
                "Psl\\Vec\\filter_with_keys",
                "Psl\\Vec\\filter_nulls",
                "Psl\\Dict\\filter",
                "Psl\\Dict\\filter_keys",
                "Psl\\Dict\\filter_with_keys",
                "Psl\\Dict\\filter_nulls",
            ],
        ),
        ("array_flip", vec!["Psl\\Dict\\flip"]),
        ("array_find", vec!["Psl\\Iter\\search", "Psl\\Iter\\search_with_keys", "Psl\\Iter\\search_keys"]),
        ("array_intersect", vec!["Psl\\Dict\\intersect"]),
        ("array_intersect_key", vec!["Psl\\Dict\\intersect_by_key"]),
        ("array_key_exists", vec!["Psl\\Iter\\contains_key"]),
        ("array_key_first", vec!["Psl\\Iter\\first_key"]),
        ("array_key_last", vec!["Psl\\Iter\\last_key"]),
        ("array_keys", vec!["Psl\\Vec\\keys"]),
        (
            "array_map",
            vec![
                "Psl\\Vec\\map",
                "Psl\\Vec\\map_with_key",
                "Psl\\Dict\\map",
                "Psl\\Dict\\map_keys",
                "Psl\\Dict\\map_with_key",
            ],
        ),
        ("array_merge", vec!["Psl\\Vec\\concat", "Psl\\Dict\\merge"]),
        ("array_rand", vec!["Psl\\Iter\\random"]),
        ("array_reduce", vec!["Psl\\Iter\\reduce", "Psl\\Iter\\reduce_keys", "Psl\\Iter\\reduce_with_keys"]),
        ("array_reverse", vec!["Psl\\Vec\\reverse"]),
        (
            "array_slice",
            vec![
                "Psl\\Vec\\slice",
                "Psl\\Vec\\take",
                "Psl\\Vec\\take_while",
                "Psl\\Vec\\drop",
                "Psl\\Vec\\drop_while",
                "Psl\\Dict\\slice",
                "Psl\\Dict\\take",
                "Psl\\Dict\\take_while",
                "Psl\\Dict\\drop",
                "Psl\\Dict\\drop_while",
            ],
        ),
        ("array_sum", vec!["Psl\\Math\\sum", "Psl\\Math\\sum_floats"]),
        (
            "array_unique",
            vec![
                "Psl\\Vec\\unique",
                "Psl\\Vec\\unique_by",
                "Psl\\Vec\\unique_scalar",
                "Psl\\Dict\\unique",
                "Psl\\Dict\\unique_by",
                "Psl\\Dict\\unique_scalar",
            ],
        ),
        ("array_walk", vec!["Psl\\Iter\\apply"]),
        ("uasort", vec!["Psl\\Dict\\sort", "Psl\\Dict\\sort_by", "Psl\\Vec\\sort_by"]),
        ("asort", vec!["Psl\\Dict\\sort", "Psl\\Dict\\sort_by", "Psl\\Vec\\sort_by"]),
        ("uksort", vec!["Psl\\Dict\\sort_by_key"]),
        ("ksort", vec!["Psl\\Dict\\sort_by_key"]),
        ("usort", vec!["Psl\\Vec\\sort", "Psl\\Vec\\sort_by"]),
        ("sort", vec!["Psl\\Vec\\sort", "Psl\\Vec\\sort_by"]),
        ("array_values", vec!["Psl\\Vec\\values"]),
        ("sizeof", vec!["Psl\\Iter\\count"]),
        ("count", vec!["Psl\\Iter\\count"]),
        ("in_array", vec!["Psl\\Iter\\contains"]),
        ("shuffle", vec!["Psl\\Iter\\shuffle"]),
    ])
});
