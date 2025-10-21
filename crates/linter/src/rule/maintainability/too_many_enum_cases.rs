use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMember;
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
pub struct TooManyEnumCasesRule {
    meta: &'static RuleMeta,
    cfg: TooManyEnumCasesConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TooManyEnumCasesConfig {
    pub level: Level,
    pub threshold: u16,
}

impl Default for TooManyEnumCasesConfig {
    fn default() -> Self {
        Self { level: Level::Error, threshold: 20 }
    }
}

impl Config for TooManyEnumCasesConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for TooManyEnumCasesRule {
    type Config = TooManyEnumCasesConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Too Many Enum Cases",
            code: "too-many-enum-cases",
            description: indoc::indoc! {r#"
                Detects enums with too many cases.

                This rule checks the number of cases in enums. If the number of cases exceeds a configurable threshold, an issue is reported.
            "#},
            good_example: indoc::indoc! {r#"
                <?php

                enum SimpleEnum {
                    case A;
                    case B;
                    case C;
                }
            "#},
            bad_example: indoc::indoc! {r#"
                <?php

                enum LargeEnum {
                    case A;
                    case B;
                    case C;
                    case D;
                    case E;
                    case F;
                    case G;
                    case H;
                    case I;
                    case J;
                    case K;
                    case L;
                    case M;
                    case N;
                    case O;
                    case P;
                    case Q;
                    case R;
                    case S;
                    case T;
                    case U;
                }
            "#},
            category: Category::Maintainability,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Enum];

        TARGETS
    }

    fn build(settings: &RuleSettings<TooManyEnumCasesConfig>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let r#enum = match node {
            Node::Enum(e) => e,
            _ => return,
        };

        let mut cases = 0;
        for member in r#enum.members.iter() {
            if let ClassLikeMember::EnumCase(_) = member {
                cases += 1;
            }
        }

        if cases > self.cfg.threshold {
            ctx.collector.report(
                Issue::new(self.cfg.level, "Enum has too many cases.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(r#enum.span()).with_message(format!(
                            "Enum has {cases} cases, which exceeds the threshold of {}.",
                            self.cfg.threshold
                        ))
                    )
                    .with_note("Large enums can be difficult to read, reason about, or maintain.")
                    .with_help(
                        "Try splitting the enum into smaller logical groups or refactoring to reduce the total number of cases."
                    )
            );
        }
    }
}
