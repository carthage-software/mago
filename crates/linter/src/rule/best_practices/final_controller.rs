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
pub struct FinalControllerRule {
    meta: &'static RuleMeta,
    cfg: FinalControllerConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct FinalControllerConfig {
    pub level: Level,
}

impl Default for FinalControllerConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for FinalControllerConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for FinalControllerRule {
    type Config = FinalControllerConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Final Controller",
            code: "final-controller",
            description: indoc! {r#"
                Enforces that controller classes are declared as `final`.

                In modern MVC frameworks, controllers should be treated as entry points that orchestrate the application's response to a request. They are not designed to be extension points.

                Extending controllers can lead to deep inheritance chains, making the codebase rigid and difficult to maintain. It's a best practice to favor composition (injecting services for shared logic) over inheritance.

                If a controller is intended as a base for others, it should be explicitly marked as `abstract`. All other concrete controllers should be `final` to prevent extension.
            "#},
            good_example: indoc! {r#"
                <?php

                namespace App\Http\Controllers;

                final class UserController
                {
                    // ...
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                namespace App\Http\Controllers;

                class UserController
                {
                    // ...
                }
            "#},
            category: Category::BestPractices,
            requirements: RuleRequirements::Any(&[
                RuleRequirements::Integration(Integration::Symfony),
                RuleRequirements::Integration(Integration::Laravel),
                RuleRequirements::Integration(Integration::Tempest),
                RuleRequirements::Integration(Integration::Spiral),
                RuleRequirements::Integration(Integration::CakePHP),
                RuleRequirements::Integration(Integration::Yii),
            ]),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Class];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Class(class) = node else {
            return;
        };

        if class.modifiers.contains_abstract() || class.modifiers.contains_final() {
            return;
        }

        if !class.name.value.ends_with("Controller") {
            return;
        }

        let issue = Issue::new(self.cfg.level, "Controller classes should be declared as `final`.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(class.name.span()).with_message("Consider making this class `final` or `abstract`"),
            )
            .with_note("If this is a concrete controller, declare it as `final`.")
            .with_note("If this is a base class for other controllers, declare it as `abstract`.")
            .with_note("For shared logic, prefer composition (injecting services) over inheritance.")
            .with_help("Add the `final` keyword to the class declaration.");

        ctx.collector.report(issue);
    }
}
