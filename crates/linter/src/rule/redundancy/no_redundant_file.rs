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
pub struct NoRedundantFileRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantFileConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantFileConfig {
    pub level: Level,
}

impl Default for NoRedundantFileConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantFileConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantFileRule {
    type Config = NoRedundantFileConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant File",
            code: "no-redundant-file",
            description: indoc! {"
                Detects redundant files that contain no executable code or declarations.
                "},
            good_example: indoc! {r#"
                <?php

                declare(strict_types=1);

                function foo(): void {
                    return 42;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                declare(strict_types=1);
                // This file is redundant.
            "#},
            category: Category::Redundancy,

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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        let has_useful_statements = program.statements.iter().any(|statement| is_statement_useful(statement));
        if !has_useful_statements {
            let issue = Issue::new(self.cfg.level(), "Redundant file with no executable code or declarations.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(program.span())
                        .with_message("This file contains no executable code or declarations"),
                )
                .with_help("Remove the file to simplify the project.");

            ctx.collector.report(issue);
        }
    }
}

#[inline]
fn is_statement_useful<'ast, 'arena>(statement: &'ast Statement<'arena>) -> bool {
    match statement {
        Statement::Inline(inline) => !inline.value.trim().is_empty(),
        Statement::Namespace(namespace) => {
            let statements = namespace.statements().as_slice();

            statements.iter().any(|statement| is_statement_useful(statement))
        }
        Statement::Block(block) => {
            let statements = block.statements.as_slice();

            statements.iter().any(|statement| is_statement_useful(statement))
        }
        Statement::Declare(declare) => match &declare.body {
            DeclareBody::Statement(statement) => is_statement_useful(statement),
            DeclareBody::ColonDelimited(declare_colon_delimited_body) => {
                let statements = declare_colon_delimited_body.statements.as_slice();

                statements.iter().any(|statement| is_statement_useful(statement))
            }
        },
        Statement::Try(r#try) => {
            r#try.block.statements.iter().any(|statement| is_statement_useful(statement))
                || r#try
                    .catch_clauses
                    .iter()
                    .any(|catch| catch.block.statements.iter().any(|statement| is_statement_useful(statement)))
                || r#try
                    .finally_clause
                    .iter()
                    .any(|finally| finally.block.statements.iter().any(|statement| is_statement_useful(statement)))
        }
        Statement::Expression(expression_statement) => is_expression_useful(expression_statement.expression),
        Statement::Foreach(_)
        | Statement::For(_)
        | Statement::While(_)
        | Statement::DoWhile(_)
        | Statement::Continue(_)
        | Statement::Break(_)
        | Statement::Switch(_)
        | Statement::If(_) => true,
        Statement::Echo(_) | Statement::HaltCompiler(_) | Statement::Unset(_) => true,
        Statement::Class(_) | Statement::Interface(_) | Statement::Trait(_) | Statement::Enum(_) => true,
        Statement::Constant(_) | Statement::Function(_) => true,
        Statement::Return(_) => true,
        _ => false,
    }
}

#[inline]
fn is_expression_useful(expression: &Expression) -> bool {
    match expression {
        Expression::Binary(binary) => is_expression_useful(binary.lhs) || is_expression_useful(binary.rhs),
        Expression::UnaryPrefix(unary_prefix) => is_expression_useful(unary_prefix.operand),
        Expression::UnaryPostfix(unary_postfix) => is_expression_useful(unary_postfix.operand),
        Expression::Parenthesized(parenthesized) => is_expression_useful(parenthesized.expression),
        Expression::Literal(_) => false,
        Expression::MagicConstant(_) => false,
        Expression::Variable(_) => false,
        Expression::Array(array) => array.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(el) => is_expression_useful(el.key) || is_expression_useful(el.value),
            ArrayElement::Value(el) => is_expression_useful(el.value),
            ArrayElement::Variadic(el) => is_expression_useful(el.value),
            ArrayElement::Missing(_) => false,
        }),
        Expression::List(list) => list.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(el) => is_expression_useful(el.key) || is_expression_useful(el.value),
            ArrayElement::Value(el) => is_expression_useful(el.value),
            ArrayElement::Variadic(el) => is_expression_useful(el.value),
            ArrayElement::Missing(_) => false,
        }),
        Expression::LegacyArray(array) => array.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(el) => is_expression_useful(el.key) || is_expression_useful(el.value),
            ArrayElement::Value(el) => is_expression_useful(el.value),
            ArrayElement::Variadic(el) => is_expression_useful(el.value),
            ArrayElement::Missing(_) => false,
        }),
        _ => true,
    }
}
