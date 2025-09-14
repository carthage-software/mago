use indoc::indoc;
use mago_fixer::SafetyClassification;
use mago_span::HasSpan;
use mago_span::Span;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantUseRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantUseConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantUseConfig {
    pub level: Level,
}

impl Default for NoRedundantUseConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoRedundantUseConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantUseRule {
    type Config = NoRedundantUseConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Use",
            code: "no-redundant-use",
            description: indoc! {"
                Detects `use` statements that import items that are never used.
            "},
            good_example: indoc! {r#"
                <?php
                namespace App;

                use App\Helpers\ArrayHelper;

                $result = ArrayHelper::combine([]);
            "#},
            bad_example: indoc! {r#"
                <?php
                namespace App;

                use App\Helpers\ArrayHelper;
                use App\Helpers\StringHelper;

                $result = ArrayHelper::combine([]);
            "#},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Use];
        TARGETS
    }

    fn build(settings: RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Use(use_statement) = node else {
            return;
        };

        if is_entire_statement_unused(ctx, &use_statement.items) {
            let issue = Issue::new(self.cfg.level(), "Unused import statement.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(use_statement.span()).with_message("This import is never used"))
                .with_help("Remove the unused import statement.");

            ctx.collector.propose(issue, |plan| {
                plan.delete(use_statement.span().to_range(), SafetyClassification::Safe);
            });
        } else {
            for (name, alias, span) in get_unused_items(ctx, &use_statement.items) {
                let message = if let Some(alias) = alias {
                    format!("Unused import: `{} as {}`", name, alias)
                } else {
                    format!("Unused import: `{}`", name)
                };

                let issue = Issue::new(self.cfg.level(), "Unused import statement.")
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(span).with_message(&message))
                    .with_help("Remove the unused import from the group import statement.");

                ctx.collector.propose(issue, |plan| {
                    plan.delete(span.to_range(), SafetyClassification::Safe);
                });
            }
        }
    }
}

fn is_entire_statement_unused(ctx: &LintContext<'_, '_>, items: &UseItems) -> bool {
    utils::get_use_items(items).all(|item| !utils::is_name_used(ctx, item))
}

fn get_unused_items<'arena>(
    ctx: &LintContext<'_, 'arena>,
    items: &'arena UseItems<'arena>,
) -> Vec<(&'arena str, Option<&'arena str>, Span)> {
    utils::get_use_items(items).filter_map(|item| utils::get_unused_item_info(ctx, item)).collect()
}

mod utils {
    use crate::context::LintContext;
    use mago_span::{HasSpan, Span};
    use mago_syntax::ast::*;

    pub fn get_use_items<'arena>(
        items: &'arena UseItems<'arena>,
    ) -> Box<dyn Iterator<Item = &'arena UseItem<'arena>> + 'arena> {
        match items {
            UseItems::Sequence(sequence) => Box::new(sequence.items.nodes.iter()),
            UseItems::TypedSequence(sequence) => Box::new(sequence.items.nodes.iter()),
            UseItems::TypedList(list) => Box::new(list.items.nodes.iter()),
            UseItems::MixedList(list) => Box::new(list.items.nodes.iter().map(|typed_item| &typed_item.item)),
        }
    }

    #[inline]
    pub fn get_unused_item_info<'arena>(
        ctx: &LintContext<'_, 'arena>,
        item: &UseItem<'arena>,
    ) -> Option<(&'arena str, Option<&'arena str>, Span)> {
        if is_name_used(ctx, item) {
            return None;
        }

        let name = extract_short_name(item.name.value());
        let alias = item.alias.as_ref().map(|a| a.identifier.value);
        Some((name, alias, item.span()))
    }

    #[inline]
    fn extract_short_name<'a>(name: &'a str) -> &'a str {
        name.rsplit('\\').next().unwrap_or(name)
    }

    #[inline]
    pub fn is_name_used(ctx: &LintContext<'_, '_>, item: &UseItem) -> bool {
        let identifier = item
            .alias
            .as_ref()
            .map(|alias| alias.identifier.value)
            .unwrap_or_else(|| extract_short_name(item.name.value()));

        for (_position, (resolved_name, _was_imported)) in ctx.resolved_names.all().iter() {
            if *resolved_name == identifier || extract_short_name(resolved_name) == identifier {
                return true;
            }
        }

        false
    }
}
