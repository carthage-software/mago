use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::ClassLikeMember;
use mago_syntax::cst::ModifierSequenceExt;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoLegacyConstructorRule {
    meta: &'static RuleMeta,
    cfg: NoLegacyConstructorConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoLegacyConstructorConfig {
    pub level: Level,
}

impl Default for NoLegacyConstructorConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoLegacyConstructorConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoLegacyConstructorRule {
    type Config = NoLegacyConstructorConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Legacy Constructor",
            code: "no-legacy-constructor",
            description: "Detects class-named methods that are no longer invoked as constructors in PHP 8 and later.",
            good_example: indoc! {r"
                <?php

                class LegacyItem {
                    public function __construct() {}
                }
            "},
            bad_example: indoc! {r"
                <?php

                class LegacyItem {
                    public function LegacyItem() {}
                }
            "},
            category: Category::Correctness,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP80)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Class];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Class(class) = node else {
            return;
        };

        if ctx.php_version < PHPVersion::PHP80 || !ctx.scope.get_namespace().is_empty() {
            return;
        }

        if class.members.iter().any(|member| {
            matches!(member, ClassLikeMember::Method(method) if method.name.value.eq_ignore_ascii_case(b"__construct"))
        }) {
            return;
        }

        for member in &class.members {
            let ClassLikeMember::Method(method) = member else {
                continue;
            };

            if method.modifiers.contains_static() || !method.name.value.eq_ignore_ascii_case(class.name.value) {
                continue;
            }

            let issue = Issue::new(self.cfg.level(), "Class-named methods are not constructors in PHP 8 and later.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(method.name.span()).with_message("This method is not invoked by `new`"),
                )
                .with_annotation(Annotation::secondary(class.name.span()).with_message("Class declared here"))
                .with_note("PHP 8 treats PHP-4-style constructors as ordinary methods, which can skip initialization.")
                .with_help("Rename this method to `__construct` if it is intended to initialize new instances.");

            ctx.collector.report(issue);
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use mago_php_version::PHPVersion;
    use mago_reporting::Level;

    use super::NoLegacyConstructorRule;
    use crate::registry::RuleRegistry;
    use crate::rule::tests::collect_issues;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = zero_argument_constructor,
        rule = NoLegacyConstructorRule,
        count = 1,
        settings = |s: &mut Settings| s.php_version = PHPVersion::PHP83,
        code = indoc! {r"
            <?php
            declare(strict_types=1);

            final class LegacyItem {
                public int $value = 0;
                public function LegacyItem() { $this->value = 123; }
            }
            echo (new LegacyItem())->value;
        "}
    }

    test_lint_failure! {
        name = case_insensitive_constructor_with_arguments,
        rule = NoLegacyConstructorRule,
        count = 1,
        settings = |s: &mut Settings| s.php_version = PHPVersion::PHP80,
        code = "<?php class LegacyItem { public function legacyITEM($value) {} }"
    }

    test_lint_failure! {
        name = explicit_global_namespace,
        rule = NoLegacyConstructorRule,
        count = 2,
        code = indoc! {r"
            <?php
            namespace Example {
                class Item { public function Item() {} }
            }
            namespace {
                class First { public function First() {} }
            }
            namespace Other {
                class Item { public function Item() {} }
            }
            namespace {
                class Last { public function Last() {} }
            }
        "}
    }

    test_lint_success! {
        name = namespaced_class,
        rule = NoLegacyConstructorRule,
        code = "<?php namespace Example; class Item { public function Item() {} }"
    }

    test_lint_success! {
        name = modern_constructor,
        rule = NoLegacyConstructorRule,
        code = indoc! {r"
            <?php
            class Item {
                public function Item() {}
                public function __CONSTRUCT() { $this->Item(); }
            }
        "}
    }

    test_lint_success! {
        name = non_constructor_methods,
        rule = NoLegacyConstructorRule,
        code = indoc! {r"
            <?php
            class Item {
                public static function Item() {}
                public function ItemFactory() {}
            }
            trait LegacyTrait { public function LegacyTrait() {} }
            interface LegacyInterface { public function LegacyInterface(); }
            enum LegacyEnum { public function LegacyEnum() {} }
            $item = new class { public function Item() {} };
        "}
    }

    test_lint_success! {
        name = php74_constructor,
        rule = NoLegacyConstructorRule,
        settings = |s: &mut Settings| s.php_version = PHPVersion::PHP74,
        code = "<?php class Item { public function Item() {} }"
    }

    #[test]
    fn enabled_by_default_only_for_php8_and_later() {
        for (php_version, expected) in
            [(PHPVersion::PHP74, false), (PHPVersion::PHP80, true), (PHPVersion::PHP83, true)]
        {
            let settings = Settings { php_version, ..Settings::default() };
            let registry = RuleRegistry::build(&settings, None, false);
            assert_eq!(registry.is_rule_enabled("no-legacy-constructor"), expected);
        }
    }

    #[test]
    fn warning_without_automatic_rename() {
        let issues = collect_issues::<NoLegacyConstructorRule, fn(&mut Settings)>(
            "<?php class Item { public function Item() {} }",
            None,
            None,
        );

        assert_eq!(issues.len(), 1);
        let Some(issue) = issues.iter().next() else {
            panic!("Expected a legacy constructor warning");
        };
        assert_eq!(issue.level, Level::Warning);
        assert_eq!(issue.code.as_deref(), Some("no-legacy-constructor"));
        assert!(issues.to_edit_batches().is_empty());
    }
}
