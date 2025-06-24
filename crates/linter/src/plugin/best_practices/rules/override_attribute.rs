use indoc::indoc;

use mago_codex::*;
use mago_fixer::SafetyClassification;
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
pub struct OverrideAttributeRule;

impl Rule for OverrideAttributeRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Override Attribute", Level::Error)
            .with_minimum_supported_php_version(PHPVersion::PHP83)
            .with_description(indoc! {"
                Ensures proper usage of the #[Override] attribute in PHP code.

                Validates three main scenarios:

                1. Overriding methods must have the #[Override] attribute
                2. Non-overriding methods must NOT have the attribute
                3. Constructors cannot use the #[Override] attribute

                Helps prevent subtle inheritance bugs and improves code clarity.
            "})
            .with_example(RuleUsageExample::valid(
                "Correct #[Override] usage",
                indoc! {r#"
                    <?php

                    class ParentClass {
                        protected function process(): void {}
                    }

                    class ChildClass extends ParentClass {
                        #[Override]
                        protected function process(): void {}
                    }
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Correct #[Override] usage in interface implementation",
                indoc! {r#"
                    <?php

                    interface Processor {
                        protected function processSomething(): void;
                    }

                    final class ProcessorImpl implements Processor {
                        #[Override]
                        protected function processSomething(): void {
                            // Implementation
                        }
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Missing #[Override] attribute in overriding method",
                indoc! {r#"
                    <?php

                    class ParentClass {
                        public function save(): void {}
                    }

                    class ChildClass extends ParentClass {
                        public function save(): void {}  // Missing attribute
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Missing #[Override] attribute in interface implementation",
                indoc! {r#"
                    <?php

                    interface Processor {
                        protected function processSomething(): void;
                    }

                    final class ProcessorImpl implements Processor {
                        protected function processSomething(): void {}
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Unnecessary #[Override] attribute in non-overriding method",
                indoc! {r#"
                    <?php

                    class Example {
                        #[Override]
                        public function uniqueMethod(): void {}  // Not overriding
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Invalid #[Override] attribute in constructor",
                indoc! {r#"
                    <?php

                    class ParentClass {
                        public function __construct() {}
                    }

                    class ChildClass extends ParentClass {
                        #[Override]
                        public function __construct() {}  // Constructors cannot use the attribute
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let (metadata, name_id, members) = match node {
            Node::Class(class) => {
                let name = context.resolved_names.get(&class.name);
                let Some(metadata) = get_class(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                (metadata, Some(name), class.members.iter())
            }
            Node::Enum(r#enum) => {
                let name = context.resolved_names.get(&r#enum.name);
                let Some(metadata) = get_enum(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                (metadata, Some(name), r#enum.members.iter())
            }
            Node::Interface(r#interface) => {
                let name = context.resolved_names.get(&r#interface.name);
                let Some(metadata) = get_interface(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                (metadata, Some(name), r#interface.members.iter())
            }
            Node::Trait(r#trait) => {
                let name = context.resolved_names.get(&r#trait.name);
                let Some(metadata) = get_trait(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                (metadata, Some(name), r#trait.members.iter())
            }
            Node::AnonymousClass(anonymous_class) => {
                let Some(metadata) = get_anonymous_class(context.codebase, context.interner, anonymous_class.span())
                else {
                    return LintDirective::default();
                };

                (metadata, None, anonymous_class.members.iter())
            }
            _ => return LintDirective::default(),
        };

        let class_name = match name_id {
            Some(name) => context.interner.lookup(name),
            None => context.interner.lookup(&metadata.original_name),
        };

        for member in members {
            let ClassLikeMember::Method(method) = member else {
                continue;
            };

            let (override_attribute, attribute_list_index) = 'outer: {
                for (index, attribute_list) in method.attribute_lists.iter().enumerate() {
                    for attribute in attribute_list.attributes.iter() {
                        let name = context.lookup_name(&attribute.name);

                        if name.eq_ignore_ascii_case("Override") {
                            break 'outer (Some(attribute), index);
                        }
                    }
                }

                (None, 0)
            };

            let name = context.interner.lookup(&method.name.value);
            if name.eq_ignore_ascii_case("__construct") {
                if let Some(attribute) = override_attribute {
                    let issue = Issue::new(context.level(), "Invalid `#[Override]` attribute on constructor.")
                        .with_annotation(
                            Annotation::primary(attribute.span())
                                .with_message("Constructors cannot be marked with `#[Override]`."),
                        )
                        .with_note("PHP constructors don't override parent constructors.")
                        .with_help("Remove the `#[Override]` attribute from the constructor.");

                    context.propose(issue, |plan| {
                        let attribute_list = &method.attribute_lists.as_slice()[attribute_list_index];
                        if attribute_list.attributes.len() == 1 {
                            plan.delete(attribute_list.span().to_range(), SafetyClassification::Safe);
                        } else {
                            plan.delete(attribute.span().to_range(), SafetyClassification::Safe);
                        }
                    });
                }

                continue;
            }

            let lowercase_name = context.interner.lowered(&method.name.value);
            let Some(parent_class_names) = metadata.get_overridden_method_id(&lowercase_name) else {
                if let Some(attribute) = override_attribute {
                    let issue = Issue::new(
                        context.level(),
                        format!("Unnecessary `#[Override]` attribute on `{class_name}::{name}`."),
                    )
                    .with_annotation(
                        Annotation::primary(attribute.span())
                            .with_message("This method doesn't override any parent method."),
                    )
                    .with_note("The attribute should only be used when explicitly overriding a parent method.")
                    .with_help(format!("Remove the `#[Override]` attribute from `{name}` or verify inheritance."));

                    context.propose(issue, |plan| {
                        let attribute_list = &method.attribute_lists.as_slice()[attribute_list_index];
                        if attribute_list.attributes.len() == 1 {
                            plan.delete(attribute_list.span().to_range(), SafetyClassification::Safe);
                        } else {
                            plan.delete(attribute.span().to_range(), SafetyClassification::Safe);
                        }
                    });
                }

                continue;
            };

            if override_attribute.is_some() {
                continue;
            }

            let Some(parents_metadata) = parent_class_names
                .iter()
                .filter_map(|parent_class| get_class_like(context.codebase, context.interner, parent_class))
                .next()
            else {
                continue;
            };

            let parent_classname = context.interner.lookup(&parents_metadata.original_name);

            let issue = Issue::new(
                context.level(),
                format!("Missing `#[Override]` attribute on overriding method `{class_name}::{name}`."),
            )
            .with_annotation(
                Annotation::primary(method.name.span)
                    .with_message(format!("This method overrides `{parent_classname}::{name}`.")),
            )
            .with_note("The `#[Override]` attribute clarifies intent and prevents accidental signature mismatches.")
            .with_help("Add `#[Override]` attribute to method declaration.");

            context.propose(issue, |plan| {
                let code = context.interner.lookup(&context.source.content);

                let offset = method.span().start.offset;
                let line_start_offset =
                    context.source.get_line_start_offset(context.source.line_number(offset)).unwrap_or(offset);

                let indent =
                    code[line_start_offset..offset].chars().take_while(|c| c.is_whitespace()).collect::<String>();

                plan.insert(method.span().start.offset, format!("#[\\Override]\n{indent}"), SafetyClassification::Safe);
            });
        }

        LintDirective::default()
    }
}
