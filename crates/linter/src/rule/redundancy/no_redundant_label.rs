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
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantLabelRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantLabelConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantLabelConfig {
    pub level: Level,
}

impl Default for NoRedundantLabelConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantLabelConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantLabelRule {
    type Config = NoRedundantLabelConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Label",
            code: "no-redundant-label",
            description: indoc! {"
                Detects redundant `goto` labels that are declared but not used.
            "},
            good_example: indoc! {r#"
                <?php

                goto end;
                echo "Hello, world!";
                end:
            "#},
            bad_example: indoc! {r#"
                <?php

                label:
                echo "Hello, world!";
            "#},
            category: Category::Redundancy,

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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Program(_) = node else {
            return;
        };

        let labels =
            node.filter_map(
                |node| {
                    if let Node::Label(label) = node { Some((label.name.value, label.span())) } else { None }
                },
            );

        let gotos = node.filter_map(|node| if let Node::Goto(goto) = node { Some(goto.label.value) } else { None });

        for (label_name, label_span) in labels.into_iter() {
            if gotos.contains(&label_name) {
                continue;
            }

            let issue = Issue::new(self.cfg.level(), format!("Redundant goto label `{}`.", label_name))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(label_span).with_message("This label is declared but not used"))
                .with_note(format!("Label `{}` is declared but not used by any `goto` statement.", label_name))
                .with_help("Remove the redundant label.");

            ctx.collector.report(issue);
        }
    }
}
