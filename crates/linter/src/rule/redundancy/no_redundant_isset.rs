use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::isset::AccessPath;
use crate::rule::utils::isset::build_access_path;
use crate::rule::utils::isset::is_proper_prefix;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantIssetRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantIssetConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantIssetConfig {
    pub level: Level,
}

impl Default for NoRedundantIssetConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantIssetConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantIssetRule {
    type Config = NoRedundantIssetConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Isset",
            code: "no-redundant-isset",
            description: indoc! {"
                Detects redundant arguments in `isset()` calls where a nested access already implies the parent checks.

                For example, `isset($d, $d['first'], $d['first']['second'])` can be simplified to
                `isset($d['first']['second'])` because checking a nested array access or property access
                implicitly verifies that all parent levels exist.
            "},
            good_example: indoc! {r"
                <?php

                if (isset($d['first']['second'])) {
                    echo 'all present';
                }
            "},
            bad_example: indoc! {r"
                <?php

                if (isset($d, $d['first'], $d['first']['second'])) {
                    echo 'all present';
                }
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::IssetConstruct];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::IssetConstruct(construct) = node else {
            return;
        };

        let values = &construct.values;
        if values.len() < 2 {
            return;
        }

        let paths: Vec<Option<AccessPath<'_>>> = values.iter().map(|expr| build_access_path(expr)).collect();

        let mut redundant = vec![false; values.len()];
        for i in 0..values.len() {
            let Some(path_i) = &paths[i] else {
                continue;
            };

            for (j, path_j) in paths.iter().enumerate() {
                if i == j {
                    continue;
                }

                let Some(path_j) = path_j else {
                    continue;
                };

                if is_proper_prefix(path_i, path_j) {
                    redundant[i] = true;
                    break;
                }
            }

            if !redundant[i] {
                for path_j in paths.iter().skip(i + 1) {
                    if path_j.as_ref().is_some_and(|path_j| path_j == path_i) {
                        redundant[i] = true;
                        break;
                    }
                }
            }
        }

        if !redundant.iter().any(|&r| r) {
            return;
        }

        let mut annotations = Vec::new();
        for (i, expr) in values.iter().enumerate() {
            if redundant[i] {
                annotations.push(
                    Annotation::primary(expr.span())
                        .with_message("This check is redundant because a more specific check already covers it."),
                );
            }
        }

        let mut issue = Issue::new(self.cfg.level(), "Redundant `isset` argument(s) detected.")
            .with_code(self.meta.code)
            .with_help("Remove the redundant arguments; the more specific checks already imply the broader ones.");

        for annotation in annotations {
            issue = issue.with_annotation(annotation);
        }

        let nodes = &values.nodes;
        let tokens = &values.tokens;

        ctx.collector.propose(issue, |edits| {
            let mut i = 0;
            while i < nodes.len() {
                if !redundant[i] {
                    i += 1;
                    continue;
                }

                let group_start = i;
                while i < nodes.len() && redundant[i] {
                    i += 1;
                }

                let group_end = i;
                let range = if group_end < nodes.len() {
                    nodes[group_start].start_offset()..nodes[group_end].start_offset()
                } else {
                    tokens[group_start - 1].start.offset..nodes[group_end - 1].end_offset()
                };

                edits.push(TextEdit::delete(range));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = single_arg,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($a);"
    }

    test_lint_success! {
        name = unrelated_args,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($a, $b);"
    }

    test_lint_success! {
        name = sibling_accesses,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($d['a'], $d['b']);"
    }

    test_lint_success! {
        name = different_roots,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($a['x'], $b['x']);"
    }

    test_lint_success! {
        name = different_property_chains,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($a->x, $a->y);"
    }

    test_lint_success! {
        name = dynamic_property_not_compared,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($a->{$x}, $a->{$x}->y);"
    }

    test_lint_success! {
        name = variable_array_index_not_compared,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($a[$x], $a[$x]['y']);"
    }

    test_lint_failure! {
        name = integer_index_chain,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($a[0], $a[0][1]);"
    }

    test_lint_failure! {
        name = simple_chain,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($d, $d['first']);"
    }

    test_lint_failure! {
        name = deep_chain,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($d, $d['first'], $d['first']['second']);"
    }

    test_lint_failure! {
        name = reverse_order,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($d['first']['second'], $d['first'], $d);"
    }

    test_lint_failure! {
        name = property_chain,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($o, $o->foo);"
    }

    test_lint_failure! {
        name = mixed_chain,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($d, $d['first']['second']);"
    }

    test_lint_failure! {
        name = duplicate,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($d, $d);"
    }

    test_lint_failure! {
        name = mixed_property_and_array,
        rule = NoRedundantIssetRule,
        code = r"<?php isset($foo, $foo->bar, $foo->bar->baz, $foo->bar->baz['x'], $foo->bar->baz['x']['y']);"
    }
}
