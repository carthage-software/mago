use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct KanDefectRule {
    meta: &'static RuleMeta,
    cfg: KanDefectConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct KanDefectConfig {
    pub level: Level,
    pub threshold: f64,
}

impl Default for KanDefectConfig {
    fn default() -> Self {
        Self { level: Level::Error, threshold: 1.6 }
    }
}

impl Config for KanDefectConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for KanDefectRule {
    type Config = KanDefectConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Kan Defect",
            code: "kan-defect",
            description: indoc::indoc! {r#"
                Detects classes, traits, interfaces, functions, and closures with high kan defect.

                The "Kan Defect" metric is a heuristic for estimating defect proneness in a class or similar structure.
                It counts control-flow statements (`while`, `do`, `foreach`, `if`, and `switch`) and sums them using a
                formula loosely based on the work of Stephen H. Kan.

                References:
                  - https://github.com/phpmetrics/PhpMetrics/blob/c43217cd7783bbd54d0b8c1dd43f697bc36ef79d/src/Hal/Metric/Class_/Complexity/KanDefectVisitor.php
                  - https://phpmetrics.org/
            "#},
            good_example: indoc::indoc! {r#"
                <?php

                function handleRequest($request) {
                    $validated = validateRequest($request);
                    $processed = processRequest($validated);
                    return formatResponse($processed);
                }

                function validateRequest($request) {
                    if (empty($request['type'])) {
                        return null;
                    }
                    return $request;
                }

                function processRequest($request) {
                    return match($request['type']) {
                        'create' => createResource($request),
                        'update' => updateResource($request),
                        'delete' => deleteResource($request),
                        default => null
                    };
                }

                function formatResponse($data) {
                    return ['status' => 'success', 'data' => $data];
                }
            "#},
            bad_example: indoc::indoc! {r#"
                <?php

                function handleRequest($request) {
                    if (empty($request)) {
                        return null;
                    }

                    if (!isset($request['type'])) {
                        return null;
                    }

                    switch ($request['type']) {
                        case 'create':
                            if (!isset($request['data'])) {
                                return null;
                            }
                            break;
                        case 'update':
                            if (!isset($request['id'])) {
                                return null;
                            }
                            break;
                        case 'delete':
                            if (!isset($request['id'])) {
                                return null;
                            }
                            break;
                    }

                    if (isset($request['filters'])) {
                        foreach ($request['filters'] as $key => $value) {
                            switch ($key) {
                                case 'status':
                                    if ($value === 'active') {
                                        // filter
                                    }
                                    break;
                                case 'category':
                                    if (!empty($value)) {
                                        // filter
                                    }
                                    break;
                            }
                        }
                    }

                    while (!empty($request['items'])) {
                        $item = array_shift($request['items']);
                        if ($item['valid']) {
                            foreach ($item['tags'] as $tag) {
                                if ($tag === 'important') {
                                    // process
                                }
                            }
                        }
                    }

                    foreach ($request['metadata'] as $meta) {
                        switch ($meta['type']) {
                            case 'timestamp':
                                break;
                            case 'user':
                                break;
                        }
                    }

                    return ['status' => 'success'];
                }
            "#},
            category: Category::Maintainability,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Class,
            NodeKind::Trait,
            NodeKind::AnonymousClass,
            NodeKind::Enum,
            NodeKind::Interface,
            NodeKind::Function,
            NodeKind::Closure,
        ];
        TARGETS
    }

    fn build(settings: &RuleSettings<KanDefectConfig>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let kind = match node.kind() {
            NodeKind::Class => "Class",
            NodeKind::Trait => "Trait",
            NodeKind::AnonymousClass => "Class",
            NodeKind::Enum => "Enum",
            NodeKind::Interface => "Interface",
            NodeKind::Function => "Function",
            NodeKind::Closure => "Closure",
            _ => return,
        };

        let threshold = self.cfg.threshold;
        let kan_defect = get_kan_defect_of_node(node);

        if kan_defect > threshold {
            ctx.collector.report(
                Issue::new(self.cfg.level, format!("{kind} has a high kan defect score ({kan_defect})."))
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(node.span()).with_message(format!(
                        "{kind} has a kan defect score of {kan_defect}, which exceeds the threshold of {threshold}.",
                    )))
                    .with_note("Kan defect is a heuristic used by phpmetrics to estimate defect-proneness based on control-flow statements.")
                    .with_help("Try reducing the number of loops, switch statements, or if statements.")
                    .with_help("You can also consider splitting large units of code into smaller, more focused units.")
            );
        }
    }
}

#[inline]
fn get_kan_defect_of_node<'ast, 'arena>(node: Node<'ast, 'arena>) -> f64 {
    let (select_count, while_count, if_count) = collect_defect_factors(node);

    calculate_kan_defect(select_count, while_count, if_count)
}

#[inline]
const fn calculate_kan_defect(select: usize, r#while: usize, r#if: usize) -> f64 {
    0.15 + 0.23 * (r#while as f64) + 0.22 * (select as f64) + 0.07 * (r#if as f64)
}

#[inline]
fn collect_defect_factors<'ast, 'arena>(node: Node<'ast, 'arena>) -> (usize, usize, usize) {
    let mut select_count = 0;
    let mut while_count = 0;
    let mut if_count = 0;

    for child in node.children() {
        let (s, w, i) = collect_defect_factors(child);
        select_count += s;
        while_count += w;
        if_count += i;
    }

    match node {
        Node::Switch(_) | Node::Match(_) => select_count += 1,
        Node::DoWhile(_) | Node::While(_) | Node::Foreach(_) => while_count += 1,
        Node::If(_) => if_count += 1,
        _ => (),
    }

    (select_count, while_count, if_count)
}
