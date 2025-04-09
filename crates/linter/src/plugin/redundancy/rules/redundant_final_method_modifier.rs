use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantFinalMethodModifierRule;

impl Rule for RedundantFinalMethodModifierRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant Final Method Modifier", Level::Help)
            .with_description(indoc! {"
                Detects redundant `final` modifiers on methods in final classes or enum methods.
            "})
            .with_example(RuleUsageExample::invalid(
                "A redundant `final` modifier on a method in a final class",
                indoc! {r#"
                    <?php

                    final class Foo
                    {
                        final public function bar(): void
                        {
                            // ...
                        }
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A redundant `final` modifier on a method in an enum",
                indoc! {r#"
                    <?php

                    enum Foo
                    {
                        case Bar;
                        case Baz;

                        final public function qux(): void
                        {
                            // ...
                        }
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let (members, is_enum) = match node {
            Node::Class(class) => {
                if !class.modifiers.contains_final() {
                    return LintDirective::default();
                }

                (&class.members, false)
            }
            Node::Enum(r#enum) => (&r#enum.members, true),
            _ => return LintDirective::default(),
        };

        if !members.contains_methods() {
            return LintDirective::Prune;
        }

        for member in members.iter() {
            if let ClassLikeMember::Method(method) = member {
                let Some(final_modifier) = method.modifiers.get_final() else {
                    continue;
                };

                let method_name = context.interner.lookup(&method.name.value);

                let message = if is_enum {
                    format!(
                        "The `final` modifier on enum method `{}` is redundant as enums cannot be extended.",
                        method_name,
                    )
                } else {
                    format!(
                        "The `final` modifier on method `{}` is redundant as the class is already final.",
                        method_name,
                    )
                };

                let issue = Issue::new(context.level(), message)
                    .with_annotations([
                        Annotation::primary(final_modifier.span()).with_message("This `final` modifier is redundant.")
                    ])
                    .with_help("Remove the redundant `final` modifier.");

                context
                    .propose(issue, |plan| plan.delete(final_modifier.span().to_range(), SafetyClassification::Safe));
            }
        }

        LintDirective::default()
    }
}
