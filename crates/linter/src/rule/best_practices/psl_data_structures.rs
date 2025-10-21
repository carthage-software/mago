use std::sync::LazyLock;

use ahash::HashMap;
use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PslDataStructuresRule {
    meta: &'static RuleMeta,
    cfg: PslDataStructuresConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PslDataStructuresConfig {
    pub level: Level,
}

impl Default for PslDataStructuresConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PslDataStructuresConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PslDataStructuresRule {
    type Config = PslDataStructuresConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Psl Data Structures",
            code: "psl-data-structures",
            description: indoc! {"
                This rule enforces the usage of Psl data structures over their SPL counterparts.

                Psl data structures are preferred because they are type-safe and provide more consistent behavior.
            "},
            good_example: indoc! {r#"
                <?php

                declare(strict_types=1);

                use Psl\DataStructure\Stack;

                $stack = new Stack();
            "#},
            bad_example: indoc! {r#"
                <?php

                declare(strict_types=1);

                $stack = new SplStack();
            "#},
            category: Category::BestPractices,

            requirements: RuleRequirements::Integration(Integration::Psl),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Instantiation];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Instantiation(instantiation) = node else {
            return;
        };
        let Expression::Identifier(identifier) = instantiation.class else {
            return;
        };

        let class_name = ctx.lookup_name(identifier).to_lowercase();
        if let Some(replacements) = DATA_STRUCTURE_REPLACEMENTS.get(class_name.as_str()) {
            ctx.collector.report(
                Issue::new(
                    self.cfg.level(),
                    "Use the Psl data structure instead of the SPL counterpart.",
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(identifier.span())
                        .with_message("This is an SPL data structure"),
                )
                .with_note("Psl data structures are preferred because they are type-safe and provide more consistent behavior.")
                .with_help(format!(
                    "Use `{}` instead.",
                    format_replacements(replacements),
                )),
            );
        }
    }
}

fn format_replacements(replacements: &[&str]) -> String {
    let mut result = String::new();
    for (i, replacement) in replacements.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        result.push_str(replacement);
    }
    result
}

static DATA_STRUCTURE_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("splstack", vec!["Psl\\DataStructure\\Stack"]),
        ("splqueue", vec!["Psl\\DataStructure\\Queue", "Psl\\DataStructure\\PriorityQueue"]),
    ])
});
