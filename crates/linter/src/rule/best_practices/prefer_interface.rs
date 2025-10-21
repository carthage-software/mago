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
pub struct PreferInterfaceRule {
    meta: &'static RuleMeta,
    cfg: PreferInterfaceConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreferInterfaceConfig {
    pub level: Level,
}

impl Default for PreferInterfaceConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for PreferInterfaceConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferInterfaceRule {
    type Config = PreferInterfaceConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Interface",
            code: "prefer-interface",
            description: indoc! {"
                Detects when an implementation class is used instead of the interface.
            "},
            good_example: indoc! {r#"
                <?php

                use Symfony\Component\Serializer\SerializerInterface;

                class UserController
                {
                    public function __construct(SerializerInterface $serializer)
                    {
                        $this->serializer = $serializer;
                    }
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                use Symfony\Component\Serializer\Serializer;

                class UserController
                {
                    public function __construct(Serializer $serializer)
                    {
                        $this->serializer = $serializer;
                    }
                }
            "#},
            category: Category::BestPractices,

            requirements: RuleRequirements::Integration(Integration::Symfony),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Hint];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Hint(Hint::Identifier(identifier)) = node else {
            return;
        };

        let fqcn = ctx.lookup_name(identifier);
        for (implementation, interface) in IMPLEMENTATION_TO_INTERFACE.iter() {
            if fqcn == *implementation {
                let issue = Issue::new(
                    self.cfg.level(),
                    format!("Use the interface `{}` instead of the implementation `{}`.", interface, implementation),
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(identifier.span())
                        .with_message("This uses the implementation instead of the interface"),
                );

                ctx.collector.report(issue);

                return;
            }
        }
    }
}

const IMPLEMENTATION_TO_INTERFACE: [(&str, &str); 3] = [
    ("Symfony\\Component\\Serializer\\Serializer", "Symfony\\Component\\Serializer\\SerializerInterface"),
    (
        "Symfony\\Component\\Serializer\\Encoder\\JsonEncode",
        "Symfony\\Component\\Serializer\\Encoder\\DecoderInterface",
    ),
    (
        "Symfony\\Component\\Serializer\\Encoder\\JsonDecode",
        "Symfony\\Component\\Serializer\\Encoder\\DecoderInterface",
    ),
];
