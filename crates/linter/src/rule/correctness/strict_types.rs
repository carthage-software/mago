use indoc::indoc;
use mago_fixer::SafetyClassification;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
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

const STRICT_TYPES_DIRECTIVE: &str = "strict_types";

#[derive(Debug, Clone)]
pub struct StrictTypesRule {
    meta: &'static RuleMeta,
    cfg: StrictTypesConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct StrictTypesConfig {
    pub level: Level,
    #[serde(alias = "allow-disabling")]
    pub allow_disabling: bool,
}

impl Default for StrictTypesConfig {
    fn default() -> Self {
        Self { level: Level::Warning, allow_disabling: false }
    }
}

impl Config for StrictTypesConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for StrictTypesRule {
    type Config = StrictTypesConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Strict Types",
            code: "strict-types",
            description: indoc! {"
                Detects missing `declare(strict_types=1);` statement at the beginning of the file.
            "},
            good_example: indoc! {r###"
                <?php

                declare(strict_types=1);

                echo "Hello, World!";
            "###},
            bad_example: indoc! {r###"
                <?php

                echo "Hello, World!";
            "###},
            category: Category::Correctness,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP70)),
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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        let mut found = false;
        let mut has_useful_statements = false;
        for statement in program.statements.iter() {
            let declare = match statement {
                Statement::Declare(declare) => declare,
                _ => {
                    has_useful_statements |= !matches!(
                        statement,
                        Statement::OpeningTag(_) | Statement::ClosingTag(_) | Statement::Inline(_) | Statement::Noop(_)
                    );

                    continue;
                }
            };

            for item in declare.items.iter() {
                if item.name.value != STRICT_TYPES_DIRECTIVE {
                    continue;
                }

                match &item.value {
                    Expression::Literal(Literal::Integer(integer)) => {
                        if integer.value == Some(0) && !self.cfg.allow_disabling {
                            let issue = Issue::new(self.cfg.level(), "The `strict_types` directive is disabled.")
                                .with_code(self.meta.code)
                                .with_annotation(
                                    Annotation::primary(item.span())
                                        .with_message("The `strict_types` is disabled here"),
                                )
                                .with_note("Disabling `strict_types` can lead to type safety issues.")
                                .with_help("Consider setting `strict_types` to `1` to enforce strict typing.");

                            ctx.collector.report(issue);
                        }
                    }
                    _ => {
                        // ignore other values, as they will be caught by the semantics checker
                    }
                };

                found = true;
            }
        }

        if !has_useful_statements {
            // empty file or file with only opening/closing tags, inline HTML, or no-op statements
            return;
        }

        if !found {
            let issue = Issue::new(
                self.cfg.level(),
                "Missing `declare(strict_types=1);` statement at the beginning of the file.",
            )
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(program.span()))
            .with_note("The `strict_types` directive enforces strict type checking, which can prevent subtle bugs.")
            .with_help("Add `declare(strict_types=1);` at the top of your file.");

            ctx.collector.propose(issue, |plan| {
                let Some(mut first_statement) = program.statements.first() else {
                    // The file is completely empty, insert an opening tag and declare statement
                    // This change is safe because the file is empty.
                    plan.insert(0, "<?php\n\ndeclare(strict_types=1);\n", SafetyClassification::Safe);

                    return;
                };

                // If the first statement is a shebang.
                if let Statement::Inline(Inline { kind: InlineKind::Shebang, value, span, .. }) = first_statement {
                    // Skip the shebang and look for the first PHP statement.
                    first_statement = match program.statements.get(1) {
                        Some(statement) => statement,
                        None => {
                            let ends_in_newline = value.ends_with('\n');

                            // If there are no statements after the shebang, insert an opening tag and declare statement.
                            let content = if ends_in_newline {
                                "<?php\n\ndeclare(strict_types=1);\n"
                            } else {
                                "\n<?php\n\ndeclare(strict_types=1);\n"
                            };

                            // This is safe because the shebang is the only statement in the file.
                            plan.insert(span.end.offset, content, SafetyClassification::Safe);

                            return;
                        }
                    };
                }

                match first_statement {
                    Statement::Inline(inline) => {
                        // If the first statement is an inline statement, insert the declare statement before it.
                        let starts_with_newline = inline.value.starts_with('\n');
                        let content = if !starts_with_newline {
                            "<?php\n\ndeclare(strict_types=1);\n\n?>\n"
                        } else {
                            "<?php\n\ndeclare(strict_types=1);\n\n?>"
                        };

                        plan.insert(inline.span.start.offset, content, SafetyClassification::PotentiallyUnsafe);
                    }
                    Statement::OpeningTag(
                        OpeningTag::Full(FullOpeningTag { span, .. }) | OpeningTag::Short(ShortOpeningTag { span }),
                    ) => {
                        // If the first statement is an opening tag, insert the declare statement after it.
                        plan.insert(
                            span.end.offset,
                            "\n\ndeclare(strict_types=1);\n",
                            SafetyClassification::PotentiallyUnsafe,
                        );
                    }
                    Statement::EchoTag(echo_tag) => {
                        // If the first statement is an echo opening tag, insert an opening tag and declare statement
                        // and a closing tag before it.
                        plan.insert(
                            echo_tag.tag.start.offset,
                            "<?php\n\ndeclare(strict_types=1);\n\n?>\n",
                            SafetyClassification::PotentiallyUnsafe,
                        );
                    }
                    _ => unreachable!(),
                }
            });
        }
    }
}
