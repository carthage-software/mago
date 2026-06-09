use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Modifier;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Property;
use mago_syntax::ast::PropertyItem;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferCastsMethodRule {
    meta: &'static RuleMeta,
    cfg: PreferCastsMethodConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct PreferCastsMethodConfig {
    pub level: Level,
}

impl Default for PreferCastsMethodConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for PreferCastsMethodConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferCastsMethodRule {
    type Config = PreferCastsMethodConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Casts Method",
            code: "prefer-casts-method",
            description: indoc! {"
                Detects the `$casts` property on an Eloquent model. Laravel 11 introduced the `casts()`
                method as the modern way to declare attribute casts, allowing the cast values to be
                expressed with code (such as enum or class references) rather than a static array.
            "},
            good_example: indoc! {r"
                <?php

                class User
                {
                    protected function casts(): array
                    {
                        return [
                            'is_admin' => 'boolean',
                        ];
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                class User
                {
                    protected $casts = [
                        'is_admin' => 'boolean',
                    ];
                }
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::Laravel),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Property];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Property(Property::Plain(property)) = node else {
            return;
        };

        // A `casts()` method is never static, so a static `$casts` is left alone.
        if property.modifiers.iter().any(|modifier| matches!(modifier, Modifier::Static(_))) {
            return;
        }

        for item in property.items.iter() {
            let PropertyItem::Concrete(concrete) = item else {
                continue;
            };

            // Property names are case-sensitive in PHP.
            if concrete.variable.name != b"$casts" {
                continue;
            }

            if !matches!(concrete.value, Expression::Array(_) | Expression::LegacyArray(_)) {
                continue;
            }

            let issue = Issue::new(
                self.cfg.level(),
                "Define model casts in the `casts()` method instead of the `$casts` property.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(concrete.variable.span())
                    .with_message("This `$casts` property should be a `casts()` method"),
            )
            .with_note("Laravel 11 introduced the `casts()` method as the modern way to declare attribute casts.")
            .with_help(
                "Move the cast definitions into a `protected function casts(): array` method that returns the array.",
            );

            ctx.collector.report(issue);

            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = casts_property,
        rule = PreferCastsMethodRule,
        code = indoc! {r"
            <?php

            class User
            {
                protected $casts = [
                    'options' => 'array',
                ];
            }
        "}
    }

    test_lint_success! {
        name = casts_method,
        rule = PreferCastsMethodRule,
        code = indoc! {r"
            <?php

            class User
            {
                protected function casts(): array
                {
                    return ['options' => 'array'];
                }
            }
        "}
    }

    test_lint_success! {
        name = other_property,
        rule = PreferCastsMethodRule,
        code = indoc! {r"
            <?php

            class User
            {
                protected $fillable = ['name', 'email'];
            }
        "}
    }

    test_lint_success! {
        name = casts_without_array_value,
        rule = PreferCastsMethodRule,
        code = indoc! {r"
            <?php

            class User
            {
                protected $casts;
            }
        "}
    }

    test_lint_success! {
        name = static_casts_left_alone,
        rule = PreferCastsMethodRule,
        code = indoc! {r"
            <?php

            class Registry
            {
                protected static $casts = ['a' => 'b'];
            }
        "}
    }
}
