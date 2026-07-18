use indoc::indoc;
use regex::bytes::Regex;
use schemars::JsonSchema;

use mago_allocator::Arena;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::comments::docblock::get_docblock_for_node;
use mago_syntax::cst::ClassLikeMember;
use mago_syntax::cst::DirectVariable;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Program;
use mago_syntax::cst::Property;
use mago_syntax::cst::Statement;

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
    excludes: Box<CompiledExcludes>,
}

#[derive(Debug, Clone, Eq, PartialEq, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
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
    /// Per-declaration-kind regular expressions that exempt matching names from
    /// requiring a docblock. A declaration is exempt only when *every* name it
    /// declares matches at least one pattern for its kind.
    pub exclude: MissingDocsExclude,
}

/// Regular expressions, grouped by declaration kind, that exempt matching names
/// from the missing-docs check.
///
/// Defaults to empty lists for every kind, so existing configurations that do
/// not set `exclude` behave exactly as before.
#[derive(Debug, Clone, Default, Eq, PartialEq, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct MissingDocsExclude {
    pub functions: Vec<String>,
    pub classes: Vec<String>,
    pub interfaces: Vec<String>,
    pub traits: Vec<String>,
    pub enums: Vec<String>,
    pub enum_cases: Vec<String>,
    pub constants: Vec<String>,
    pub statics: Vec<String>,
    pub methods: Vec<String>,
    pub properties: Vec<String>,
}

/// Compiled counterpart of [`MissingDocsExclude`], built once when the rule is
/// constructed so patterns are not recompiled per declaration.
#[derive(Debug, Clone, Default)]
struct CompiledExcludes {
    functions: Vec<Regex>,
    classes: Vec<Regex>,
    interfaces: Vec<Regex>,
    traits: Vec<Regex>,
    enums: Vec<Regex>,
    enum_cases: Vec<Regex>,
    constants: Vec<Regex>,
    statics: Vec<Regex>,
    methods: Vec<Regex>,
    properties: Vec<Regex>,
}

impl CompiledExcludes {
    fn from_config(exclude: &MissingDocsExclude) -> Self {
        Self {
            functions: compile_patterns(&exclude.functions),
            classes: compile_patterns(&exclude.classes),
            interfaces: compile_patterns(&exclude.interfaces),
            traits: compile_patterns(&exclude.traits),
            enums: compile_patterns(&exclude.enums),
            enum_cases: compile_patterns(&exclude.enum_cases),
            constants: compile_patterns(&exclude.constants),
            statics: compile_patterns(&exclude.statics),
            methods: compile_patterns(&exclude.methods),
            properties: compile_patterns(&exclude.properties),
        }
    }
}

/// Compiles a list of pattern strings, silently dropping any that fail to parse
/// so an invalid user-supplied regex never aborts the lint run.
fn compile_patterns(patterns: &[String]) -> Vec<Regex> {
    patterns.iter().filter_map(|pattern| Regex::new(pattern).ok()).collect()
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
            exclude: MissingDocsExclude::default(),
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
        let excludes = Box::new(CompiledExcludes::from_config(&settings.config.exclude));

        Self { meta: Self::meta(), cfg: settings.config.clone(), excludes }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Program(program) = node else {
            return;
        };

        for stmt in &program.statements {
            self.check_statement(ctx, program, stmt);
        }
    }
}

impl MissingDocsRule {
    fn check_statement<'ast, 'arena, A>(
        &self,
        ctx: &mut LintContext<'_, 'arena, A>,
        program: &'ast Program<'arena>,
        stmt: &'ast Statement<'arena>,
    ) where
        A: Arena,
    {
        match stmt {
            Statement::Function(func) if self.cfg.functions => {
                let names = [func.name.value];
                self.check_docs(ctx, program, func, "function", &names, &self.excludes.functions);
            }
            Statement::Namespace(ns) => {
                for inner_stmt in ns.statements() {
                    self.check_statement(ctx, program, inner_stmt);
                }
            }
            Statement::Class(class) => {
                if self.cfg.classes {
                    let names = [class.name.value];
                    self.check_docs(ctx, program, class, "class", &names, &self.excludes.classes);
                }

                self.check_members(ctx, program, class.members.iter());
            }
            Statement::Interface(interface) => {
                if self.cfg.interfaces {
                    let names = [interface.name.value];
                    self.check_docs(ctx, program, interface, "interface", &names, &self.excludes.interfaces);
                }

                self.check_members(ctx, program, interface.members.iter());
            }
            Statement::Trait(tr) => {
                if self.cfg.traits {
                    let names = [tr.name.value];
                    self.check_docs(ctx, program, tr, "trait", &names, &self.excludes.traits);
                }

                self.check_members(ctx, program, tr.members.iter());
            }
            Statement::Enum(en) => {
                if self.cfg.enums {
                    let names = [en.name.value];
                    self.check_docs(ctx, program, en, "enum", &names, &self.excludes.enums);
                }

                self.check_members(ctx, program, en.members.iter());
            }
            Statement::Constant(constant) if self.cfg.constants => {
                let names: Vec<_> = constant.items.iter().map(|item| item.name.value).collect();
                self.check_docs(ctx, program, constant, "constant", &names, &self.excludes.constants);
            }
            Statement::Static(static_decl) if self.cfg.statics => {
                let names: Vec<_> = static_decl.items.iter().map(|item| variable_name(item.variable())).collect();
                self.check_docs(ctx, program, static_decl, "static variable", &names, &self.excludes.statics);
            }
            _ => {}
        }
    }

    fn check_members<'ast, 'arena, A>(
        &self,
        ctx: &mut LintContext<'_, 'arena, A>,
        program: &'ast Program<'arena>,
        members: impl Iterator<Item = &'ast ClassLikeMember<'arena>>,
    ) where
        A: Arena,
    {
        for member in members {
            match member {
                ClassLikeMember::Constant(constant) if self.cfg.constants => {
                    let names: Vec<_> = constant.items.iter().map(|item| item.name.value).collect();
                    self.check_docs(ctx, program, constant, "class constant", &names, &self.excludes.constants);
                }
                ClassLikeMember::EnumCase(enum_case) if self.cfg.enum_cases => {
                    let names = [enum_case.item.name().value];
                    self.check_docs(ctx, program, enum_case, "enum case", &names, &self.excludes.enum_cases);
                }
                ClassLikeMember::Method(method) if self.cfg.methods => {
                    let names = [method.name.value];
                    self.check_docs(ctx, program, method, "method", &names, &self.excludes.methods);
                }
                ClassLikeMember::Property(prop) if self.cfg.properties => {
                    let names = property_names(prop);
                    self.check_docs(ctx, program, prop, "property", &names, &self.excludes.properties);
                }
                _ => {}
            }
        }
    }

    fn check_docs<'ast, 'arena, A>(
        &self,
        ctx: &mut LintContext<'_, 'arena, A>,
        program: &'ast Program<'arena>,
        node: &'ast impl HasSpan,
        subject: &'static str,
        names: &[&[u8]],
        excludes: &[Regex],
    ) where
        A: Arena,
    {
        if is_excluded(names, excludes) {
            return;
        }

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

/// Returns `true` when a declaration should be exempt from the missing-docs check.
///
/// A declaration is exempt only when there is at least one pattern for its kind
/// and *every* name it declares matches at least one of those patterns.
fn is_excluded(names: &[&[u8]], excludes: &[Regex]) -> bool {
    if excludes.is_empty() || names.is_empty() {
        return false;
    }

    names.iter().all(|name| excludes.iter().any(|pattern| pattern.is_match(name)))
}

/// Returns a variable's name with the leading `$` dropped, so patterns match the
/// bare name (e.g. `^get` matches `$getter`).
fn variable_name<'arena>(variable: &DirectVariable<'arena>) -> &'arena [u8] {
    variable.name.strip_prefix(b"$".as_slice()).unwrap_or(variable.name)
}

/// Collects every name declared by a property, across both plain (`$a, $b`) and
/// hooked property forms.
fn property_names<'arena>(property: &Property<'arena>) -> Vec<&'arena [u8]> {
    match property {
        Property::Plain(plain) => plain.items.iter().map(|item| variable_name(item.variable())).collect(),
        Property::Hooked(hooked) => vec![variable_name(hooked.item.variable())],
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

    test_lint_success! {
        name = excluded_function_by_prefix,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.functions = true;
            s.rules.missing_docs.config.exclude.functions = vec!["^test_".to_string(), "^get[A-Z]".to_string()];
        },
        code = indoc! {r#"
            <?php

            function test_it_works(): void {
            }

            function getName(): string {
                return 'x';
            }
        "#}
    }

    test_lint_failure! {
        name = non_excluded_function_still_reported,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.functions = true;
            s.rules.missing_docs.config.exclude.functions = vec!["^test_".to_string()];
        },
        code = indoc! {r#"
            <?php

            function compute(): void {
            }
        "#}
    }

    test_lint_success! {
        name = excluded_method_by_accessor_prefix,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.methods = true;
            s.rules.missing_docs.config.exclude.methods = vec!["^get[A-Z]".to_string(), "^set[A-Z]".to_string()];
        },
        code = indoc! {r#"
            <?php

            class Foo {
                public function getName(): string {
                    return 'x';
                }

                public function setName(string $name): void {
                }
            }
        "#}
    }

    test_lint_success! {
        name = excluded_class_by_suffix,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.classes = true;
            s.rules.missing_docs.config.exclude.classes = vec!["Test$".to_string()];
        },
        code = indoc! {r#"
            <?php

            class FooTest {
            }
        "#}
    }

    test_lint_success! {
        name = excluded_property_matches_bare_name,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.properties = true;
            s.rules.missing_docs.config.exclude.properties = vec!["^cached".to_string()];
        },
        code = indoc! {r#"
            <?php

            class Foo {
                public string $cachedValue;
            }
        "#}
    }

    test_lint_success! {
        name = excluded_multi_name_constant_all_match,
        rule = MissingDocsRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.constants = true;
            s.rules.missing_docs.config.exclude.constants = vec!["^INTERNAL_".to_string()];
        },
        code = indoc! {r#"
            <?php

            const INTERNAL_A = 1, INTERNAL_B = 2;
        "#}
    }

    test_lint_failure! {
        name = multi_name_constant_partial_match_still_reported,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.constants = true;
            s.rules.missing_docs.config.exclude.constants = vec!["^INTERNAL_".to_string()];
        },
        code = indoc! {r#"
            <?php

            const INTERNAL_A = 1, PUBLIC_B = 2;
        "#}
    }

    test_lint_failure! {
        name = invalid_exclude_pattern_is_ignored,
        rule = MissingDocsRule,
        count = 1,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.missing_docs.config.reset();
            s.rules.missing_docs.config.functions = true;
            s.rules.missing_docs.config.exclude.functions = vec!["(unclosed".to_string()];
        },
        code = indoc! {r#"
            <?php

            function compute(): void {
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
