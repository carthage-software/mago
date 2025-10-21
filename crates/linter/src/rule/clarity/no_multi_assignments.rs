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
pub struct NoMultiAssignmentsRule {
    meta: &'static RuleMeta,
    cfg: NoMultiAssignmentsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoMultiAssignmentsConfig {
    pub level: Level,
}

impl Default for NoMultiAssignmentsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoMultiAssignmentsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoMultiAssignmentsRule {
    type Config = NoMultiAssignmentsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Multi Assignments",
            code: "no-multi-assignments",
            description: indoc! {r#"
                Flags any instances of multiple assignments in a single statement. This can lead to
                confusion and unexpected behavior, and is generally considered poor practice.
            "#},
            good_example: indoc! {r#"
                <?php

                $b = 0;
                $a = $b;
            "#},
            bad_example: indoc! {r#"
                <?php

                $a = $b = 0;
            "#},
            category: Category::Clarity,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Assignment];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Assignment(assignment) = node else {
            return;
        };

        let Expression::Assignment(other_assignment) = assignment.rhs else {
            return;
        };

        let a = &ctx.source_file.contents[assignment.lhs.span().to_range_usize()];
        let b = &ctx.source_file.contents[other_assignment.lhs.span().to_range_usize()];
        let c = &ctx.source_file.contents[other_assignment.rhs.span().to_range_usize()];

        let issue = Issue::new(self.cfg.level, "Avoid multiple assignments in a single statement.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(assignment.span()).with_message("This chained assignment can be confusing"),
            )
            .with_note("Multiple assignments can obscure the author's intent and may be confused with a comparison.")
            .with_help(format!(
                "Did you mean `{a} = ({b} == {c})`? If not, split this into separate assignment statements for clarity."
            ));

        ctx.collector.report(issue);
    }
}
