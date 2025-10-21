use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
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
use crate::rule::utils::security::is_password;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct SensitiveParameterRule {
    meta: &'static RuleMeta,
    cfg: SensitiveParameterConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct SensitiveParameterConfig {
    pub level: Level,
}

impl Default for SensitiveParameterConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for SensitiveParameterConfig {
    fn default_enabled() -> bool {
        // TODO(azjezz): enable in the next major release
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for SensitiveParameterRule {
    type Config = SensitiveParameterConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Sensitive Parameter",
            code: "sensitive-parameter",
            description: indoc! {r#"
                Requires that parameters that are likely to contain sensitive information (e.g., passwords)
                are marked with the `#[SensitiveParameter]` attribute to prevent accidental logging or exposure.
            "#},
            good_example: indoc! {r#"
                <?php

                function login(string $username, #[SensitiveParameter] string $password): void {
                   // ...
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function login(string $username, string $password): void {
                   // ...
                }
            "#},
            category: Category::Security,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionLikeParameter];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::FunctionLikeParameter(parameter) = node else {
            return;
        };

        if !is_password(parameter.variable.name) {
            return; // Not a password-related parameter, no issue
        }

        for attribute_list in parameter.attribute_lists.iter() {
            for attribute in attribute_list.attributes.iter() {
                let name = ctx.resolved_names.get(&attribute.name);

                if name.eq_ignore_ascii_case("SensitiveParameter") {
                    return; // Attribute found, no issue
                }
            }
        }

        let issue = Issue::new(self.cfg.level(), "Parameters that may contain sensitive information should be marked with the `#[SensitiveParameter]` attribute.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(parameter.variable.span).with_message("Sensitive parameter found here."))
            .with_note("Marking sensitive parameters helps prevent accidental logging or exposure of sensitive data in exception backtraces.")
            .with_help("Add the `#[SensitiveParameter]` attribute to the parameter declaration.");

        ctx.collector.propose(issue, |plan| {
            let start_position = parameter.start_position();

            plan.insert(start_position.offset, "#[\\SensitiveParameter] ", SafetyClassification::Safe);
        });
    }
}
