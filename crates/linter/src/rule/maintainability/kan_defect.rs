use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::cst::DoWhile;
use mago_syntax::cst::Foreach;
use mago_syntax::cst::If;
use mago_syntax::cst::Match;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Switch;
use mago_syntax::cst::While;
use mago_syntax::walker::Walker;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::misc::get_class_like_header_span;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct KanDefectRule {
    meta: &'static RuleMeta,
    cfg: KanDefectConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
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
            good_example: indoc::indoc! {r"
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
            "},
            bad_example: indoc::indoc! {r"
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
            "},
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let mut factors = (0, 0, 0);
        let kind = match node {
            Node::Class(class) => {
                DefectFactorCollector.walk_class(class, &mut factors);
                "Class"
            }
            Node::Trait(r#trait) => {
                DefectFactorCollector.walk_trait(r#trait, &mut factors);
                "Trait"
            }
            Node::AnonymousClass(class) => {
                DefectFactorCollector.walk_anonymous_class(class, &mut factors);
                "Class"
            }
            Node::Enum(r#enum) => {
                DefectFactorCollector.walk_enum(r#enum, &mut factors);
                "Enum"
            }
            Node::Interface(interface) => {
                DefectFactorCollector.walk_interface(interface, &mut factors);
                "Interface"
            }
            Node::Function(function) => {
                DefectFactorCollector.walk_function(function, &mut factors);
                "Function"
            }
            Node::Closure(closure) => {
                DefectFactorCollector.walk_closure(closure, &mut factors);
                "Closure"
            }
            _ => return,
        };

        let threshold = self.cfg.threshold;
        let kan_defect = calculate_kan_defect(factors.0, factors.1, factors.2);

        if kan_defect > threshold {
            ctx.collector.report(
                Issue::new(self.cfg.level, format!("{kind} has a high kan defect score."))
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(get_class_like_header_span(node)).with_message(format!(
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
fn calculate_kan_defect(select: usize, r#while: usize, r#if: usize) -> f64 {
    0.07f64.mul_add(r#if as f64, 0.22f64.mul_add(select as f64, 0.23f64.mul_add(r#while as f64, 0.15)))
}

struct DefectFactorCollector;

impl<'ast, 'arena> Walker<'ast, 'arena, (usize, usize, usize)> for DefectFactorCollector {
    fn walk_in_switch(&self, _: &'ast Switch<'arena>, counts: &mut (usize, usize, usize)) {
        counts.0 += 1;
    }

    fn walk_in_match(&self, _: &'ast Match<'arena>, counts: &mut (usize, usize, usize)) {
        counts.0 += 1;
    }

    fn walk_in_do_while(&self, _: &'ast DoWhile<'arena>, counts: &mut (usize, usize, usize)) {
        counts.1 += 1;
    }

    fn walk_in_while(&self, _: &'ast While<'arena>, counts: &mut (usize, usize, usize)) {
        counts.1 += 1;
    }

    fn walk_in_foreach(&self, _: &'ast Foreach<'arena>, counts: &mut (usize, usize, usize)) {
        counts.1 += 1;
    }

    fn walk_in_if(&self, _: &'ast If<'arena>, counts: &mut (usize, usize, usize)) {
        counts.2 += 1;
    }
}
