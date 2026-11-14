use indoc::indoc;
use mago_allocator::Arena;
use mago_bytes::BytesDisplay;
use schemars::JsonSchema;

use foldhash::HashMap as KeyMap;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::source_text;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

mod key;

#[derive(Debug, Clone)]
pub struct DuplicateKeyRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: DuplicateKeyConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct DuplicateKeyConfig {
    pub level: Level,
}

impl Default for DuplicateKeyConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for DuplicateKeyConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DuplicateKeyRule {
    type Config = DuplicateKeyConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Duplicate Mapping Key",
            code: "twig-duplicate-key",
            description: indoc! {r"
                Reports literal mapping keys that repeat an earlier key after PHP's array-key conversion, so
                `'1'` and `1` are the same key. The last value replaces the first.

                Dynamic keys are left alone. No fix is offered, because both values can have side effects.
            "},
            good_example: indoc! {r"
                {{ {title: title, body: body} }}
            "},
            bad_example: indoc! {r"
                {{ {title: title, title: body} }}
            "},
            category: Category::Correctness,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::HashMap];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::HashMap(mapping) = node else {
            return;
        };

        if mapping.entries.len() < 2 {
            return;
        }

        let mut keys = KeyMap::default();
        for entry in mapping.entries.iter() {
            let Some(expression) = &entry.key else {
                continue;
            };

            let Some(key) = key::literal(expression) else {
                continue;
            };

            let Some(previous) = keys.insert(key, expression.span()) else {
                continue;
            };

            let span = expression.span();
            let text = BytesDisplay(source_text(ctx, span));
            ctx.collector.report(
                Issue::new(self.cfg.level, format!("The key `{text}` repeats an earlier key in this mapping."))
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(span).with_message("This value replaces the earlier one"))
                    .with_annotation(Annotation::secondary(previous).with_message("The earlier value is set here"))
                    .with_note("Twig mappings become PHP arrays, so a repeated key keeps only its last value.")
                    .with_note(
                        "PHP turns keys such as `'1'` and `true` into the integer `1`, so they collide with `1`.",
                    )
                    .with_help("Remove the entry you do not need, or give one of the two entries a different key."),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_template_1,
        rule = DuplicateKeyRule,
        code = r"{{ {a: 1, b: 2} }}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = DuplicateKeyRule,
        code = r"{{ {1: 1, '01': 2, '1.0': 3, '+1': 4} }}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = DuplicateKeyRule,
        code = r"{{ {(key): 1, other: 2} }}",
    }

    test_lint_success! {
        name = valid_template_4,
        rule = DuplicateKeyRule,
        code = r"{{ {a: 1, nested: {a: 2}} }}",
    }

    test_lint_success! {
        name = valid_template_5,
        rule = DuplicateKeyRule,
        code = r"{{ {'': 1, (+null): 2} }}",
    }

    test_lint_success! {
        name = valid_template_6,
        rule = DuplicateKeyRule,
        code = r"{{ {'01': 1, (+'01'): 2} }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = DuplicateKeyRule,
        count = 1,
        code = r"{{ {a: 1, a: 2} }}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = DuplicateKeyRule,
        count = 1,
        code = r"{{ {1: 1, '1': 2} }}",
    }

    test_lint_failure! {
        name = invalid_template_3,
        rule = DuplicateKeyRule,
        count = 2,
        code = r"{{ {(1.5): 1, (true): 2, 1: 3} }}",
    }

    test_lint_failure! {
        name = invalid_template_4,
        rule = DuplicateKeyRule,
        count = 1,
        code = r"{{ {(null): 1, '': 2} }}",
    }

    test_lint_failure! {
        name = invalid_template_5,
        rule = DuplicateKeyRule,
        count = 1,
        code = r"{{ {(-1): 1, '-1': 2} }}",
    }

    test_lint_failure! {
        name = invalid_template_6,
        rule = DuplicateKeyRule,
        count = 1,
        code = r"{{ {01: 1, 1: 2} }}",
    }

    test_lint_failure! {
        name = invalid_template_7,
        rule = DuplicateKeyRule,
        count = 1,
        code = r"{{ {'a': record(1), '\x61': record(2)} }}",
    }

    test_lint_failure! {
        name = invalid_template_8,
        rule = DuplicateKeyRule,
        count = 1,
        code = r"{{ {0: 1, (+null): 2} }}",
    }

    test_lint_failure! {
        name = invalid_template_9,
        rule = DuplicateKeyRule,
        count = 1,
        code = r"{{ {0: 1, (- -null): 2} }}",
    }
}
