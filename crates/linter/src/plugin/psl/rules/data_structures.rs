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
pub struct DataStructuresRule;

impl Rule for DataStructuresRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Data Structures", Level::Warning)
            .with_description(indoc! {"
                This rule enforces the usage of Psl data structures over their SPL counterparts.

                Psl data structures are preferred because they are type-safe and provide more consistent behavior.
            "})
            .with_example(RuleUsageExample::valid(
                "Using the `Psl\\DataStructure\\Stack` class",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    use Psl\DataStructure\Stack;

                    $stack = new Stack();
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Using the `Psl\\DataStructure\\Queue` class",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    use Psl\DataStructure\Queue;

                    $queue = new Queue();
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Using the `Psl\\DataStructure\\PriorityQueue` class",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    use Psl\DataStructure\PriorityQueue;

                    $priorityQueue = new PriorityQueue();
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using the `SplStack` class",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    $stack = new SplStack();
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using the `SplQueue` class",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    $queue = new SplQueue();
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Instantiation(instantiation) = node else { return LintDirective::default() };
        let Expression::Identifier(identifier) = instantiation.class.as_ref() else { return LintDirective::default() };

        let class_name = context.lookup_name(identifier).to_lowercase();
        if let Some(replacements) = DATA_STRUCTURE_REPLACEMENTS.get(class_name.as_str()) {
            context.report(
                Issue::new(context.level(), "Use the Psl data structure instead of the SPL counterpart.")
                    .with_annotation(Annotation::primary(identifier.span()).with_message("This is an SPL data structure."))
                    .with_note("Psl data structures are preferred because they are type-safe and provide more consistent behavior.")
                    .with_help(format!(
                        "Use `{}` instead.",
                        format_replacements(replacements),
                    )),
            );
        }

        LintDirective::default()
    }
}

static DATA_STRUCTURE_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("splstack", vec!["Psl\\DataStructure\\Stack"]),
        ("splqueue", vec!["Psl\\DataStructure\\Queue", "Psl\\DataStructure\\PriorityQueue"]),
    ])
});
