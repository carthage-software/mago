use indoc::indoc;

use mago_reporting::*;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct ParameterNameRule;

impl Rule for ParameterNameRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Parameter Name", Level::Error)
            .with_description(indoc! {"
                Ensures parameter names match their parent method declarations when overriding.
                This prevents breaking named arguments in inherited method implementations.
            "})
            .with_example(RuleUsageExample::invalid(
                "Mismatched parameter names in overriding method",
                indoc! {r#"
                    <?php

                    class ParentClass {
                        public function example($param1, $param2) {}
                    }

                    class ChildClass extends ParentClass {
                        public function example($differentName, $param2) {}
                    }
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Matching parameter names in overriding method",
                indoc! {r#"
                    <?php

                    class ParentClass {
                        public function example($param1, $param2) {}
                    }

                    class ChildClass extends ParentClass {
                        public function example($param1, $param2) {}
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let (reflection, members) = match node {
            Node::Class(class) => {
                let name = context.module.names.get(&class.name);
                let Some(reflection) = context.codebase.get_class(context.interner, name) else {
                    return LintDirective::default();
                };

                (reflection, class.members.iter())
            }
            Node::Enum(r#enum) => {
                let name = context.module.names.get(&r#enum.name);
                let Some(reflection) = context.codebase.get_enum(context.interner, name) else {
                    return LintDirective::default();
                };

                (reflection, r#enum.members.iter())
            }
            Node::Interface(r#interface) => {
                let name = context.module.names.get(&r#interface.name);
                let Some(reflection) = context.codebase.get_interface(context.interner, name) else {
                    return LintDirective::default();
                };

                (reflection, r#interface.members.iter())
            }
            Node::Trait(r#trait) => {
                let name = context.module.names.get(&r#trait.name);
                let Some(reflection) = context.codebase.get_trait(context.interner, name) else {
                    return LintDirective::default();
                };

                (reflection, r#trait.members.iter())
            }
            Node::AnonymousClass(anonymous_class) => {
                let Some(reflection) = context.codebase.get_anonymous_class(&anonymous_class) else {
                    return LintDirective::default();
                };

                (reflection, anonymous_class.members.iter())
            }
            _ => return LintDirective::default(),
        };

        for member in members {
            let ClassLikeMember::Method(method) = member else {
                continue;
            };

            let method_name = context.interner.lowered(&method.name.value);
            let method_name_str = context.interner.lookup(&method.name.value);

            if method_name_str.eq_ignore_ascii_case("__construct") {
                // Ignore constructors
                continue;
            }

            let Some(method_reflection) = reflection.methods.members.get(&method_name) else {
                continue;
            };

            if !method_reflection.is_overriding {
                continue;
            }

            let Some(parent_class_names) = reflection.methods.overriden_members.get(&method_name) else {
                continue;
            };

            let Some(parent_class_name) = parent_class_names.iter().next() else {
                continue;
            };

            let Some(parent_reflection) = context.codebase.get_class_like(parent_class_name) else {
                continue;
            };

            let Some(parent_method_reflection) = parent_reflection.methods.members.get(&method_name) else {
                continue;
            };

            for (index, parameter) in method_reflection.parameters.iter().enumerate() {
                let Some(parent_parameter) = parent_method_reflection.parameters.get(index) else {
                    continue;
                };

                let name = context.interner.lookup(&parameter.name);
                let parent_name = context.interner.lookup(&parent_parameter.name);

                if name != parent_name {
                    let current_method = method_reflection.name.get_key(context.interner);
                    let parent_method = parent_method_reflection.name.get_key(context.interner);

                    let issue = Issue::new(
                        context.level(),
                        format!("Parameter name mismatch in overridden method '{}'.", current_method),
                    )
                    .with_annotation(
                        Annotation::primary(parameter.span)
                            .with_message(format!("Parameter #{} is named `{}` in `{}`", index, name, current_method)),
                    )
                    .with_annotation(
                        Annotation::secondary(parent_parameter.span)
                            .with_message(format!("But was named `{}` in parent `{}`", parent_name, parent_method)),
                    )
                    .with_note("Changing parameter names in overridden methods can break code using named arguments.")
                    .with_note("Callers referencing parameters by name will encounter runtime errors if names differ.")
                    .with_help(format!(
                        "Rename parameter to `{}` to match the parent declaration in `{}`",
                        parent_name, parent_method
                    ));

                    context.report(issue);
                }
            }
        }

        LintDirective::default()
    }
}
