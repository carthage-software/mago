use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMember;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Program;
use mago_syntax::ast::Statement;
use mago_syntax::comments::docblock::get_docblock_for_node;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct MissingDocsRule {
    meta: &'static RuleMeta,
    cfg: MissingDocsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct MissingDocsConfig {
    pub level: Level,
    pub functions: bool,
    pub classes: bool,
    pub interfaces: bool,
    pub traits: bool,
    pub enums: bool,
    pub enum_cases: bool,
    pub constants: bool,
    pub statics: bool,
    pub methods: bool,
    pub properties: bool,
}

impl Default for MissingDocsConfig {
    fn default() -> Self {
        Self {
            level: Level::Help,
            functions: true,
            classes: false,
            interfaces: false,
            traits: false,
            enums: false,
            enum_cases: true,
            constants: true,
            statics: true,
            methods: true,
            properties: true,
        }
    }
}

impl Config for MissingDocsConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for MissingDocsRule {
    type Config = MissingDocsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Missing Docs",
            code: "missing-docs",
            description: indoc! {"
                Detects declarations that are missing a docblock.

                This rule can be configured to require documentation for functions,
                classes, interfaces, traits, enums, enum cases, constants, statics,
                methods, and properties.

                Documentation is useful when it explains intent, behaviour, usage,
                invariants, or other details that are not obvious from the code alone.
            "},
            good_example: indoc! {r#"
                <?php

                /**
                 * A helpful piece of documentation.
                 */
                function foo() {}
            "#},
            bad_example: indoc! {r#"
                <?php

                function foo() {}
            "#},
            category: Category::Clarity,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Program];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        for stmt in &program.statements {
            self.check_statement(ctx, program, stmt);
        }
    }
}

impl MissingDocsRule {
    fn check_statement<'a, 'arena>(
        &self,
        ctx: &mut LintContext<'_, 'arena>,
        program: &'a Program<'arena>,
        stmt: &'a Statement<'arena>,
    ) {
        match stmt {
            Statement::Function(func) if self.cfg.functions => {
                self.check_docs(ctx, program, func, "function");
            }
            Statement::Namespace(ns) => {
                for inner_stmt in ns.statements() {
                    self.check_statement(ctx, program, inner_stmt);
                }
            }
            Statement::Class(class) => {
                if self.cfg.classes {
                    self.check_docs(ctx, program, class, "class");
                }

                self.check_members(ctx, program, class.members.iter());
            }
            Statement::Interface(interface) => {
                if self.cfg.interfaces {
                    self.check_docs(ctx, program, interface, "interface");
                }

                self.check_members(ctx, program, interface.members.iter());
            }
            Statement::Trait(tr) => {
                if self.cfg.traits {
                    self.check_docs(ctx, program, tr, "trait");
                }

                self.check_members(ctx, program, tr.members.iter());
            }
            Statement::Enum(en) => {
                if self.cfg.enums {
                    self.check_docs(ctx, program, en, "enum");
                }

                self.check_members(ctx, program, en.members.iter());
            }
            Statement::Constant(constant) if self.cfg.constants => {
                self.check_docs(ctx, program, constant, "constant");
            }
            Statement::Static(stat) if self.cfg.statics => {
                self.check_docs(ctx, program, stat, "static variable");
            }
            _ => {}
        }
    }

    fn check_members<'a, 'arena>(
        &self,
        ctx: &mut LintContext<'_, 'arena>,
        program: &'a Program<'arena>,
        members: impl Iterator<Item = &'a ClassLikeMember<'arena>>,
    ) {
        for member in members {
            match member {
                ClassLikeMember::Constant(constant) if self.cfg.constants => {
                    self.check_docs(ctx, program, constant, "class constant");
                }
                ClassLikeMember::EnumCase(enum_case) if self.cfg.enum_cases => {
                    self.check_docs(ctx, program, enum_case, "enum case");
                }
                ClassLikeMember::Method(method) if self.cfg.methods => {
                    self.check_docs(ctx, program, method, "method");
                }
                ClassLikeMember::Property(prop) if self.cfg.properties => {
                    self.check_docs(ctx, program, prop, "property");
                }
                _ => {}
            }
        }
    }

    fn check_docs<'a, 'arena>(
        &self,
        ctx: &mut LintContext<'_, 'arena>,
        program: &'a Program<'arena>,
        node: &'a impl HasSpan,
        subject: &'static str,
    ) {
        let trivia = get_docblock_for_node(program, node);

        if trivia.is_none_or(|t| !t.kind.is_docblock()) {
            ctx.collector.report(
                Issue::new(self.cfg.level, format!("Missing docblock for {subject}"))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(node.span()).with_message(format!("This {subject} is missing a docblock")),
                    )
                    .with_help(format!("Add a docblock above this {subject}")),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::MissingDocsRule;
    use crate::rule::MissingDocsConfig;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    impl MissingDocsConfig {
        fn reset(&mut self) {
            self.functions = false;
            self.classes = false;
            self.interfaces = false;
            self.traits = false;
            self.enums = false;
            self.enum_cases = false;
            self.constants = false;
            self.statics = false;
            self.methods = false;
            self.properties = false;
        }
    }

    test_lint_failure! {
        name = function_without_docblock,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.functions = true;
        },
        code = indoc! {r#"
            <?php

            function identity(string $value): string {
                return $value;
            }
        "#}
    }

    test_lint_success! {
        name = function_with_docblock,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.functions = true;
        },
        code = indoc! {r#"
            <?php

            /**
             * This function returns the input unchanged.
             */
            function identity(string $value): string {
                return $value;
            }
        "#}
    }

    test_lint_success! {
        name = function_without_docblock_but_lint_disabled_for_functions,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.methods = true;
        },
        code = indoc! {r#"
            <?php

            function foo(string $value): void {
            }
        "#}
    }

    test_lint_failure! {
        name = namespace_function_without_docblock,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.functions = true;
        },
        code = indoc! {r#"
            <?php

            namespace App;

            function foo(): void {
            }
        "#}
    }

    test_lint_failure! {
        name = class_without_docblock,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.classes = true;
        },
        code = indoc! {r#"
            <?php

            class Foo {
            }
        "#}
    }

    test_lint_success! {
        name = class_with_docblock,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.classes = true;
        },
        code = indoc! {r#"
            <?php

            /**
             * Represents a foo.
             */
            class Foo {
            }
        "#}
    }

    test_lint_failure! {
        name = interface_without_docblock,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.interfaces = true;
        },
        code = indoc! {r#"
            <?php

            interface Foo {
            }
        "#}
    }

    test_lint_failure! {
        name = trait_without_docblock,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.traits = true;
        },
        code = indoc! {r#"
            <?php

            trait LogsMessages {
            }
        "#}
    }

    test_lint_failure! {
        name = enum_without_docblock,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.enums = true;
        },
        code = indoc! {r#"
            <?php

            enum Status {
                case Open;
            }
        "#}
    }

    test_lint_failure! {
        name = enum_without_docblock_on_cases_or_methods,
        rule = MissingDocsRule,
        count = 3,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.enum_cases = true;
            s.rules.missing_docs.config.methods = true;
        },
        code = indoc! {r#"
            <?php

            enum Foo {
                case Bar;
                case Baz;

                public function toString(): string {
                    return $this->name;
                }
            }
        "#}
    }

    test_lint_success! {
        name = enum_with_docblock_on_cases_and_methods,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.enum_cases = true;
            s.rules.missing_docs.config.methods = true;
        },
        code = indoc! {r#"
            <?php

            enum Foo {
                /**
                 * Bar case docs.
                 */
                case Bar;

                /**
                 * Baz case docs.
                 */
                case Baz;

                /**
                 * Returns the string representation.
                 */
                public function toString(): string {
                    return $this->name;
                }
            }
        "#}
    }

    test_lint_failure! {
        name = top_level_constant_without_docblock,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.constants = true;
        },
        code = indoc! {r#"
            <?php

            const FOO = 'bar';
        "#}
    }

    test_lint_failure! {
        name = class_constant_without_docblock,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.constants = true;
        },
        code = indoc! {r#"
            <?php

            class Foo {
                public const BAR = 'baz';
            }
        "#}
    }

    test_lint_success! {
        name = class_constant_without_docblock_but_constant_lint_disabled,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.methods = true;
        },
        code = indoc! {r#"
            <?php

            class Foo {
                public const BAR = 'baz';
            }
        "#}
    }

    test_lint_failure! {
        name = static_variable_without_docblock,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.statics = true;
        },
        code = indoc! {r#"
            <?php

            static $foo = 'bar';
        "#}
    }

    test_lint_failure! {
        name = method_without_docblock,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.methods = true;
        },
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar(): void {
                }
            }
        "#}
    }

    test_lint_success! {
        name = method_without_docblock_but_method_lint_disabled,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.properties = true;
        },
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar(): void {
                }
            }
        "#}
    }

    test_lint_failure! {
        name = property_without_docblock,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.properties = true;
        },
        code = indoc! {r#"
            <?php

            class Foo {
                public string $bar;
            }
        "#}
    }

    test_lint_success! {
        name = property_with_docblock,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.properties = true;
        },
        code = indoc! {r#"
            <?php

            class Foo {
                /**
                 * The bar value.
                 */
                public string $bar;
            }
        "#}
    }

    test_lint_failure! {
        name = non_docblock_comment_does_not_satisfy_rule,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.functions = true;
        },
        code = indoc! {r#"
            <?php

            // This is just a normal comment.
            function foo(): void {
            }
        "#}
    }
}
