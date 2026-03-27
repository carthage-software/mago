use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Hint;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoFullyQualifiedGlobalClassLikeRule {
    meta: &'static RuleMeta,
    cfg: NoFullyQualifiedGlobalClassLikeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoFullyQualifiedGlobalClassLikeConfig {
    pub level: Level,
}

impl Default for NoFullyQualifiedGlobalClassLikeConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoFullyQualifiedGlobalClassLikeConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl NoFullyQualifiedGlobalClassLikeRule {
    fn report_if_fq<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, identifier: Identifier<'arena>) {
        if !identifier.is_fully_qualified() {
            return;
        }

        let class_name = identifier.value().trim_start_matches('\\');
        let short_name = class_name.split('\\').next_back().unwrap_or(class_name);

        ctx.collector.report(
            Issue::new(self.cfg.level, "Fully-qualified class reference detected.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(identifier.span())
                        .with_message(format!("The reference to `\\{class_name}` uses a fully-qualified name")),
                )
                .with_note("Fully-qualified class references bypass the import system, making it harder to see which classes, interfaces, traits, and enums a file depends on.")
                .with_help(format!("Add `use {class_name};` and reference `{short_name}` directly.")),
        );
    }
}

impl LintRule for NoFullyQualifiedGlobalClassLikeRule {
    type Config = NoFullyQualifiedGlobalClassLikeConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Fully Qualified Global Class",
            code: "no-fully-qualified-global-class-like",
            description: indoc! {"
                Disallows fully-qualified class references within a namespace.

                Instead of using the backslash prefix (e.g., `new \\DateTime()` or `\\Exception`
                in a type hint), prefer an explicit `use` import statement. This applies to all
                class-like types — classes, interfaces, traits, and enums — and improves
                readability by keeping imports centralized at the top of the file.
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                use DateTime;
                use Exception;

                $dt = new DateTime();

                function foo(DateTime $dt): Exception {}
            "#},
            bad_example: indoc! {r#"
                <?php

                namespace App;

                $dt = new \DateTime();

                function foo(\DateTime $dt): \Exception {}
            "#},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Attribute,
            NodeKind::Binary,
            NodeKind::ClassConstantAccess,
            NodeKind::Extends,
            NodeKind::Hint,
            NodeKind::Implements,
            NodeKind::Instantiation,
            NodeKind::StaticMethodCall,
            NodeKind::StaticMethodPartialApplication,
            NodeKind::StaticPropertyAccess,
            NodeKind::TraitUse,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        if ctx.scope.get_namespace().is_empty() {
            return;
        }

        match node {
            Node::Attribute(attribute) => {
                self.report_if_fq(ctx, attribute.name);
            }
            Node::Extends(extends) => {
                for identifier in extends.types.iter() {
                    self.report_if_fq(ctx, *identifier);
                }
            }
            Node::Implements(implements) => {
                for identifier in implements.types.iter() {
                    self.report_if_fq(ctx, *identifier);
                }
            }
            Node::Instantiation(instantiation) => {
                let Expression::Identifier(identifier) = instantiation.class else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::StaticMethodCall(call) => {
                let Expression::Identifier(identifier) = call.class else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::StaticMethodPartialApplication(application) => {
                let Expression::Identifier(identifier) = application.class else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::StaticPropertyAccess(access) => {
                let Expression::Identifier(identifier) = access.class else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::ClassConstantAccess(access) => {
                let Expression::Identifier(identifier) = access.class else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::Binary(binary) => {
                if !matches!(binary.operator, BinaryOperator::Instanceof(_)) {
                    return;
                }
                let Expression::Identifier(identifier) = binary.rhs else {
                    return;
                };
                self.report_if_fq(ctx, *identifier);
            }
            Node::Hint(Hint::Identifier(identifier)) => {
                self.report_if_fq(ctx, *identifier);
            }
            Node::TraitUse(trait_use) => {
                for identifier in trait_use.trait_names.iter() {
                    self.report_if_fq(ctx, *identifier);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoFullyQualifiedGlobalClassLikeRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = imported_class_is_not_flagged,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            use DateTime;

            $dt = new DateTime();
        "#}
    }

    test_lint_success! {
        name = global_scope_fq_class_is_not_flagged,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            $dt = new \DateTime();
        "#}
    }

    test_lint_failure! {
        name = fq_instantiation_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $dt = new \DateTime();
        "#}
    }

    test_lint_failure! {
        name = fq_static_method_call_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $dt = \DateTime::createFromFormat('Y-m-d', '2024-01-01');
        "#}
    }

    test_lint_failure! {
        name = fq_class_constant_access_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $format = \DateTime::ATOM;
        "#}
    }

    test_lint_failure! {
        name = fq_type_hint_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            function foo(\DateTime $dt): void {}
        "#}
    }

    test_lint_failure! {
        name = fq_static_property_access_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $x = \Foo::$bar;
        "#}
    }

    test_lint_failure! {
        name = fq_first_class_callable_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $x = \Foo::bar(...);
        "#}
    }

    test_lint_failure! {
        name = fq_instanceof_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $x instanceof \Foo;
        "#}
    }

    test_lint_failure! {
        name = fq_extends_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            class Foo extends \Bar {}
        "#}
    }

    test_lint_failure! {
        name = fq_implements_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        count = 2,
        code = indoc! {r#"
            <?php

            namespace App;

            class Foo implements \Bar, \Baz {}
        "#}
    }

    test_lint_failure! {
        name = fq_attribute_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            #[\SomeAttribute]
            class Foo {}
        "#}
    }

    test_lint_failure! {
        name = fq_trait_use_in_namespace,
        rule = NoFullyQualifiedGlobalClassLikeRule,
        code = indoc! {r#"
            <?php

            namespace App;

            class Foo {
                use \SomeTrait;
            }
        "#}
    }
}
