use foldhash::HashSet;
use foldhash::HashSetExt;
use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Class;
use mago_syntax::ast::Enum;
use mago_syntax::ast::Interface;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Trait;
use mago_syntax::walker::MutWalker;
use mago_syntax::walker::walk_program_mut;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct SingleClassPerFileRule {
    meta: &'static RuleMeta,
    cfg: SingleClassPerFileConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct SingleClassPerFileConfig {
    pub level: Level,
}

impl Default for SingleClassPerFileConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for SingleClassPerFileConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for SingleClassPerFileRule {
    type Config = SingleClassPerFileConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Single Class Per File",
            code: "single-class-per-file",
            description: indoc! {"
                Ensures that each file contains at most one class-like definition (class, interface, enum, or trait).
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                class Foo
                {
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                namespace App;

                class Foo
                {
                }

                class Bar
                {
                }
            "#},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Program];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        let mut collector = ClassLikeCollector { definitions: Vec::new() };
        walk_program_mut(&mut collector, program, &mut ());

        let mut seen = HashSet::with_capacity(2);
        collector.definitions.retain(|d| seen.insert(d.1));

        if collector.definitions.len() <= 1 {
            return;
        }

        let (first_kind, first_name, first_span) = collector.definitions[0];

        for &(kind, name, span) in &collector.definitions[1..] {
            let issue = Issue::new(
                self.cfg.level(),
                format!("File contains multiple class-like definitions; found {kind} `{name}` but {first_kind} `{first_name}` is already defined in this file."),
            )
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(span).with_message(format!("{kind} `{name}` defined here")))
            .with_annotation(
                Annotation::secondary(first_span)
                    .with_message(format!("First {first_kind} `{first_name}` defined here")),
            )
            .with_help("Move each class-like definition into its own file.");

            ctx.collector.report(issue);
        }
    }
}

struct ClassLikeCollector<'arena> {
    definitions: Vec<(&'static str, &'arena str, Span)>,
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for ClassLikeCollector<'arena> {
    fn walk_in_class(&mut self, class: &'ast Class<'arena>, _: &mut ()) {
        self.definitions.push(("class", class.name.value, class.name.span()));
    }

    fn walk_in_interface(&mut self, interface: &'ast Interface<'arena>, _: &mut ()) {
        self.definitions.push(("interface", interface.name.value, interface.name.span()));
    }

    fn walk_in_trait(&mut self, r#trait: &'ast Trait<'arena>, _: &mut ()) {
        self.definitions.push(("trait", r#trait.name.value, r#trait.name.span()));
    }

    fn walk_in_enum(&mut self, r#enum: &'ast Enum<'arena>, _: &mut ()) {
        self.definitions.push(("enum", r#enum.name.value, r#enum.name.span()));
    }
}
