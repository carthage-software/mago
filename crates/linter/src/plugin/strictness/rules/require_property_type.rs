use indoc::indoc;

use mago_codex::*;
use mago_php_version::PHPVersion;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RequirePropertyTypeRule;

impl Rule for RequirePropertyTypeRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Require Property Type", Level::Warning)
            .with_description(indoc! {"
                Detects class-like properties that are missing a type hint.
            "})
            .with_minimum_supported_php_version(PHPVersion::PHP74)
            .with_example(RuleUsageExample::valid(
                "A class property that has a type hint",
                indoc! {r#"
                    <?php

                    class Foo
                    {
                        public int $bar;
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A class property that is missing a type hint",
                indoc! {r#"
                    <?php

                    class Foo
                    {
                        public $bar;
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let (metadata, members) = match node {
            Node::Class(class) => {
                let name = context.resolved_names.get(&class.name);
                let Some(metadata) = get_class(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                (metadata, class.members.as_slice())
            }
            Node::Trait(r#trait) => {
                let name = context.resolved_names.get(&r#trait.name);
                let Some(metadata) = get_trait(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                (metadata, r#trait.members.as_slice())
            }
            _ => return LintDirective::default(),
        };

        for member in members {
            let ClassLikeMember::Property(property) = member else {
                continue;
            };

            if property.hint().is_some() {
                continue;
            }

            for variable in property.variables() {
                if metadata
                    .get_appearing_property_id(&variable.name)
                    .is_none_or(|appearing_class_id| appearing_class_id != &metadata.name)
                {
                    continue;
                }

                let name = context.lookup(&variable.name);

                context.report(
                    Issue::new(context.level(), format!("Property `{name}` is missing a type hint."))
                        .with_annotation(
                            Annotation::primary(property.span())
                                .with_message(format!("Property `{name}` is declared here.")),
                        )
                        .with_note(
                            "Adding a type hint to properties improves code readability and helps prevent type errors.",
                        )
                        .with_help(format!("Consider specifying a type hint for `{name}`.")),
                );
            }
        }

        LintDirective::default()
    }
}
