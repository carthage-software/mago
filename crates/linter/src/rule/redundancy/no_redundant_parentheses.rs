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
pub struct NoRedundantParenthesesRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantParenthesesConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantParenthesesConfig {
    pub level: Level,
}

impl Default for NoRedundantParenthesesConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantParenthesesConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantParenthesesRule {
    type Config = NoRedundantParenthesesConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Parentheses",
            code: "no-redundant-parentheses",
            description: indoc! {"
                Detects redundant parentheses around expressions.
            "},
            good_example: indoc! {r#"
                <?php

                $foo = 42;
            "#},
            bad_example: indoc! {r#"
                <?php

                $foo = (42);
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Parenthesized,
            NodeKind::ExpressionStatement,
            NodeKind::PositionalArgument,
            NodeKind::NamedArgument,
            NodeKind::If,
            NodeKind::IfStatementBodyElseIfClause,
            NodeKind::IfColonDelimitedBodyElseIfClause,
            NodeKind::FunctionLikeParameterDefaultValue,
            NodeKind::EnumCaseBackedItem,
            NodeKind::PropertyConcreteItem,
            NodeKind::ConstantItem,
            NodeKind::ClassLikeConstantItem,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let parenthesized = match node {
            Node::Parenthesized(parenthesized) => {
                if let Expression::Parenthesized(inner) = parenthesized.expression {
                    inner
                } else {
                    return;
                }
            }
            Node::ExpressionStatement(expression_statement) => match expression_statement.expression {
                Expression::Parenthesized(parenthesized) => parenthesized,
                Expression::Assignment(assignment) => {
                    if let Expression::Parenthesized(rhs) = assignment.rhs {
                        if rhs.expression.is_binary() {
                            return; // Allow parentheses around binary expressions on the right-hand side of an assignment.
                        }

                        rhs
                    } else {
                        return;
                    }
                }
                _ => return,
            },
            Node::PositionalArgument(positional_argument) => {
                if positional_argument.ellipsis.is_some() {
                    return;
                }

                if let Expression::Parenthesized(value) = &positional_argument.value {
                    value
                } else {
                    return;
                }
            }
            Node::NamedArgument(named_argument) => {
                if let Expression::Parenthesized(value) = &named_argument.value {
                    value
                } else {
                    return;
                }
            }
            Node::If(r#if) => {
                if let Expression::Parenthesized(condition) = r#if.condition {
                    condition
                } else {
                    return;
                }
            }
            Node::IfStatementBodyElseIfClause(if_statement_body_else_if_clause) => {
                if let Expression::Parenthesized(condition) = if_statement_body_else_if_clause.condition {
                    condition
                } else {
                    return;
                }
            }
            Node::IfColonDelimitedBodyElseIfClause(if_colon_delimited_body_else_if_clause) => {
                if let Expression::Parenthesized(condition) = if_colon_delimited_body_else_if_clause.condition {
                    condition
                } else {
                    return;
                }
            }
            Node::FunctionLikeParameterDefaultValue(function_like_parameter_default_value) => {
                if let Expression::Parenthesized(value) = &function_like_parameter_default_value.value {
                    value
                } else {
                    return;
                }
            }
            Node::EnumCaseBackedItem(enum_case_backed_item) => {
                if let Expression::Parenthesized(value) = &enum_case_backed_item.value {
                    value
                } else {
                    return;
                }
            }
            Node::PropertyConcreteItem(property_concrete_item) => {
                if let Expression::Parenthesized(value) = &property_concrete_item.value {
                    value
                } else {
                    return;
                }
            }
            Node::ConstantItem(constant_item) => {
                if let Expression::Parenthesized(value) = &constant_item.value {
                    value
                } else {
                    return;
                }
            }
            Node::ClassLikeConstantItem(class_like_constant_item) => {
                if let Expression::Parenthesized(value) = &class_like_constant_item.value {
                    value
                } else {
                    return;
                }
            }
            _ => return,
        };

        let issue = Issue::new(self.cfg.level(), "Redundant parentheses around expression.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(parenthesized.expression.span())
                    .with_message("Expression does not need to be parenthesized"),
            )
            .with_help("Remove the redundant inner parentheses.");

        ctx.collector.report(issue);
    }
}
