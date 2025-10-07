use indoc::indoc;
use mago_fixer::SafetyClassification;
use mago_span::HasSpan;
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
    fn default_enabled() -> bool {
        // TODO(azjezz): enable this in the next major release.
        false
    }

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
        const TARGETS: &[NodeKind] = &[NodeKind::Program];
        TARGETS
    }

    fn build(settings: RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        let unused_items = utils::get_unused_items(program, ctx);

        for span in unused_items {
            let issue = Issue::new(self.cfg.level(), "Unused import.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(span).with_message("Unused import"))
                .with_help("Remove the unused import");

            ctx.collector.propose(issue, |plan| {
                plan.delete(span.to_range(), SafetyClassification::Safe);
            });
        }
    }
}

mod utils {
    use super::*;
    use std::collections::HashSet;

    pub(super) fn get_unused_items<'arena>(
        program: &'arena Program<'arena>,
        ctx: &LintContext<'_, 'arena>,
    ) -> Vec<mago_span::Span> {
        let mut unused_items = Vec::new();
        let resolved_names = get_resolved_names(ctx);

        for statement in program.statements.iter() {
            match statement {
                Statement::Use(use_stmt) => {
                    unused_items.push(use_stmt.span());
                }
                Statement::Namespace(namespace) => {
                    for ns_statement in namespace.statements().nodes.iter() {
                        if let Statement::Use(use_stmt) = ns_statement {
                            if namespace.name.is_none() {
                                unused_items.push(use_stmt.span());
                            } else {
                                let items: Vec<_> = use_items(&use_stmt.items).collect();
                                let unused: Vec<_> =
                                    items.iter().filter(|item| !is_item_used(program, item, &resolved_names)).collect();

                                if unused.len() == items.len() {
                                    unused_items.push(use_stmt.span());
                                } else {
                                    for item in unused {
                                        unused_items.push(item.span());
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        unused_items
    }

    fn use_items<'arena>(
        items: &'arena UseItems<'arena>,
    ) -> Box<dyn Iterator<Item = &'arena UseItem<'arena>> + 'arena> {
        match items {
            UseItems::Sequence(sequence) => Box::new(sequence.items.nodes.iter()),
            UseItems::TypedSequence(sequence) => Box::new(sequence.items.nodes.iter()),
            UseItems::TypedList(list) => Box::new(list.items.nodes.iter()),
            UseItems::MixedList(list) => Box::new(list.items.nodes.iter().map(|typed_item| &typed_item.item)),
        }
    }

    fn extract_simple_name(name: &str) -> &str {
        name.rsplit('\\').next().unwrap_or(name)
    }

    fn is_item_used(program: &Program, item: &UseItem, resolved_names: &HashSet<String>) -> bool {
        let identifier =
            if let Some(alias) = &item.alias { alias.identifier.value } else { extract_simple_name(item.name.value()) };

        resolved_names.contains(identifier)
            || resolved_names.contains(item.name.value())
            || is_used_in_docblocks(program, identifier)
            || is_used_in_docblocks(program, item.name.value())
    }

    fn is_used_in_docblocks(program: &Program, identifier: &str) -> bool {
        program.trivia.iter().filter(|trivia| trivia.kind.is_docblock()).any(|trivia| trivia.value.contains(identifier))
    }

    fn get_resolved_names<'arena>(ctx: &LintContext<'_, 'arena>) -> HashSet<String> {
        ctx.resolved_names
            .all()
            .iter()
            .flat_map(|(_, (resolved_name, _))| {
                let full_name = resolved_name.to_string();
                let simple_name = extract_simple_name(&full_name).to_string();
                [full_name, simple_name]
            })
            .filter(|name| !name.is_empty())
            .collect()
    }
}
