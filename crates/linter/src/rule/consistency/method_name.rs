use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_casing::is_camel_case;
use mago_casing::is_snake_case;
use mago_casing::to_camel_case;
use mago_casing::to_snake_case;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
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
pub struct MethodNameRule {
    meta: &'static RuleMeta,
    cfg: MethodNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct MethodNameConfig {
    pub level: Level,
    /// When `true`, method names must be in camelCase. When `false`, they must be in snake_case.
    pub camel: bool,
    /// When `true`, method names can be either camelCase or snake_case.
    pub either: bool,
    /// When `true`, test methods (names starting with `test`) must use snake_case
    /// (e.g., `test_something_works`), regardless of the `camel` setting.
    pub use_snake_case_for_tests: bool,
}

impl Default for MethodNameConfig {
    fn default() -> Self {
        Self { level: Level::Help, camel: true, either: false, use_snake_case_for_tests: false }
    }
}

impl Config for MethodNameConfig {
    fn level(&self) -> Level {
        self.level
    }

    fn default_enabled() -> bool {
        false
    }
}

impl LintRule for MethodNameRule {
    type Config = MethodNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Method Name",
            code: "method-name",
            description: indoc! {r#"
                Detects method declarations that do not follow the configured naming convention.

                By default, method names should be in camelCase. Magic methods (prefixed with `__`)
                are always excluded.

                The `use-snake-case-for-tests` option enforces snake_case for test methods
                (names starting with `test`), which is a common convention in PHPUnit.
            "#},
            good_example: indoc! {r"
                <?php

                class Foo
                {
                    public function getName(): string {}
                    public function setName(string $name): void {}
                }
            "},
            bad_example: indoc! {r"
                <?php

                class Foo
                {
                    public function GetName(): string {}
                    public function set_name(string $name): void {}
                }
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Method];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Method(method) = node else {
            return;
        };

        let name = method.name.value;

        // Skip magic methods (__construct, __destruct, __get, etc.)
        if name.starts_with("__") {
            return;
        }

        let is_test_method = name.starts_with("test") || name.starts_with("Test");

        // Test methods with snake_case enforcement
        if self.cfg.use_snake_case_for_tests && is_test_method {
            if !is_snake_case(name) {
                ctx.collector.report(
                    Issue::new(self.cfg.level(), format!("Test method name `{name}` should be in snake case."))
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(method.name.span())
                                .with_message(format!("Method `{name}` is declared here")),
                        )
                        .with_note(
                            "Test methods are expected to use snake_case when `use-snake-case-for-tests` is enabled.",
                        )
                        .with_help(format!(
                            "Consider renaming it to `{}` to adhere to the naming convention.",
                            to_snake_case(name)
                        )),
                );
            }

            return;
        }

        if self.cfg.either {
            if !is_camel_case(name) && !is_snake_case(name) {
                ctx.collector.report(
                    Issue::new(
                        self.cfg.level(),
                        format!("Method name `{name}` should be in either camel case or snake case."),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(method.name.span())
                            .with_message(format!("Method `{name}` is declared here")),
                    )
                    .with_note(format!(
                        "The method name `{name}` does not follow either camel case or snake naming convention."
                    ))
                    .with_help(format!(
                        "Consider renaming it to `{}` or `{}`.",
                        to_camel_case(name),
                        to_snake_case(name)
                    )),
                );
            }
        } else if self.cfg.camel {
            if !is_camel_case(name) {
                ctx.collector.report(
                    Issue::new(self.cfg.level(), format!("Method name `{name}` should be in camel case."))
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(method.name.span())
                                .with_message(format!("Method `{name}` is declared here")),
                        )
                        .with_note(format!("The method name `{name}` does not follow camel naming convention."))
                        .with_help(format!(
                            "Consider renaming it to `{}` to adhere to the naming convention.",
                            to_camel_case(name)
                        )),
                );
            }
        } else if !is_snake_case(name) {
            ctx.collector.report(
                Issue::new(self.cfg.level(), format!("Method name `{name}` should be in snake case."))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(method.name.span())
                            .with_message(format!("Method `{name}` is declared here")),
                    )
                    .with_note(format!("The method name `{name}` does not follow snake naming convention."))
                    .with_help(format!(
                        "Consider renaming it to `{}` to adhere to the naming convention.",
                        to_snake_case(name)
                    )),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::MethodNameRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = camel_case_method,
        rule = MethodNameRule,
        code = indoc! {r"
            <?php

            class Foo {
                public function getName(): void {}
                public function setName(): void {}
            }
        "}
    }

    test_lint_success! {
        name = magic_methods_excluded,
        rule = MethodNameRule,
        code = indoc! {r"
            <?php

            class Foo {
                public function __construct() {}
                public function __destruct() {}
                public function __toString(): string { return ''; }
                public function __get($name) {}
                public function __set($name, $value) {}
            }
        "}
    }

    test_lint_failure! {
        name = pascal_case_method,
        rule = MethodNameRule,
        code = indoc! {r"
            <?php

            class Foo {
                public function GetName(): void {}
            }
        "}
    }

    test_lint_failure! {
        name = snake_case_method_when_camel_required,
        rule = MethodNameRule,
        code = indoc! {r"
            <?php

            class Foo {
                public function get_name(): void {}
            }
        "}
    }

    test_lint_success! {
        name = snake_case_when_configured,
        rule = MethodNameRule,
        settings = |s: &mut crate::settings::Settings| s.rules.method_name.config.camel = false,
        code = indoc! {r"
            <?php

            class Foo {
                public function get_name(): void {}
            }
        "}
    }

    test_lint_failure! {
        name = camel_case_when_snake_required,
        rule = MethodNameRule,
        settings = |s: &mut crate::settings::Settings| s.rules.method_name.config.camel = false,
        code = indoc! {r"
            <?php

            class Foo {
                public function getName(): void {}
            }
        "}
    }

    test_lint_success! {
        name = either_camel_or_snake,
        rule = MethodNameRule,
        settings = |s: &mut crate::settings::Settings| s.rules.method_name.config.either = true,
        code = indoc! {r"
            <?php

            class Foo {
                public function getName(): void {}
                public function get_name(): void {}
            }
        "}
    }

    test_lint_failure! {
        name = neither_camel_nor_snake,
        rule = MethodNameRule,
        settings = |s: &mut crate::settings::Settings| s.rules.method_name.config.either = true,
        code = indoc! {r"
            <?php

            class Foo {
                public function Get_Name(): void {}
            }
        "}
    }

    test_lint_success! {
        name = test_method_snake_case,
        rule = MethodNameRule,
        settings = |s: &mut crate::settings::Settings| s.rules.method_name.config.use_snake_case_for_tests = true,
        code = indoc! {r"
            <?php

            class FooTest {
                public function test_something_works(): void {}
                public function test_another_thing(): void {}
                public function getName(): void {}
            }
        "}
    }

    test_lint_failure! {
        name = test_method_camel_case_when_snake_required,
        rule = MethodNameRule,
        settings = |s: &mut crate::settings::Settings| s.rules.method_name.config.use_snake_case_for_tests = true,
        code = indoc! {r"
            <?php

            class FooTest {
                public function testSomethingWorks(): void {}
            }
        "}
    }

    test_lint_failure! {
        name = multiple_bad_methods,
        rule = MethodNameRule,
        count = 2,
        code = indoc! {r"
            <?php

            class Foo {
                public function GetName(): void {}
                public function Set_Name(): void {}
            }
        "}
    }
}
