use indoc::indoc;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantFileRule;

impl Rule for RedundantFileRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant File", Level::Help)
            .with_description(indoc! {"
                Detects redundant files that contain no executable code or declarations.
            "})
            .with_example(RuleUsageExample::valid(
                "A file with a declaration.",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    function foo(): void {
                        return 42;
                    }
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "A file with executable code.",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    $x = 42;
                    $y = 42;

                    echo $x + $y;
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A redundant file.",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    // This file is redundant.
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Program(program) = node else {
            return LintDirective::default();
        };

        let has_useful_statements = program.statements.iter().any(|statement| is_statement_useful(statement, context));
        if !has_useful_statements {
            let issue = Issue::new(context.level(), "Redundant file with no executable code or declarations")
                .with_annotation(
                    Annotation::primary(program.span())
                        .with_message("This file contains no executable code or declarations."),
                )
                .with_help("Remove the file to simplify the project.");

            context.report(issue);
        }

        LintDirective::Prune
    }
}

#[inline]
fn is_statement_useful(statement: &Statement, context: &mut LintContext) -> bool {
    match statement {
        Statement::Inline(inline) => {
            let content = context.interner.lookup(&inline.value);

            !content.trim().is_empty()
        }
        Statement::Namespace(namespace) => {
            let statements = namespace.statements().as_slice();

            statements.iter().any(|statement| is_statement_useful(statement, context))
        }
        Statement::Block(block) => {
            let statements = block.statements.as_slice();

            statements.iter().any(|statement| is_statement_useful(statement, context))
        }
        Statement::Declare(declare) => match &declare.body {
            DeclareBody::Statement(statement) => is_statement_useful(statement.as_ref(), context),
            DeclareBody::ColonDelimited(declare_colon_delimited_body) => {
                let statements = declare_colon_delimited_body.statements.as_slice();

                statements.iter().any(|statement| is_statement_useful(statement, context))
            }
        },
        Statement::Try(r#try) => {
            r#try.block.statements.iter().any(|statement| is_statement_useful(statement, context))
                || r#try
                    .catch_clauses
                    .iter()
                    .any(|catch| catch.block.statements.iter().any(|statement| is_statement_useful(statement, context)))
                || r#try.finally_clause.iter().any(|finally| {
                    finally.block.statements.iter().any(|statement| is_statement_useful(statement, context))
                })
        }
        Statement::Expression(expression_statement) => is_expression_useful(&expression_statement.expression),
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
        Expression::Binary(binary) => is_expression_useful(&binary.lhs) || is_expression_useful(&binary.rhs),
        Expression::UnaryPrefix(unary_prefix) => is_expression_useful(&unary_prefix.operand),
        Expression::UnaryPostfix(unary_postfix) => is_expression_useful(&unary_postfix.operand),
        Expression::Parenthesized(parenthesized) => is_expression_useful(&parenthesized.expression),
        Expression::Literal(_) => false,
        Expression::MagicConstant(_) => false,
        Expression::Variable(_) => false,
        Expression::Array(array) => array.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(el) => is_expression_useful(&el.key) || is_expression_useful(&el.value),
            ArrayElement::Value(el) => is_expression_useful(&el.value),
            ArrayElement::Variadic(el) => is_expression_useful(&el.value),
            ArrayElement::Missing(_) => false,
        }),
        Expression::List(list) => list.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(el) => is_expression_useful(&el.key) || is_expression_useful(&el.value),
            ArrayElement::Value(el) => is_expression_useful(&el.value),
            ArrayElement::Variadic(el) => is_expression_useful(&el.value),
            ArrayElement::Missing(_) => false,
        }),
        Expression::LegacyArray(array) => array.elements.iter().any(|element| match element {
            ArrayElement::KeyValue(el) => is_expression_useful(&el.key) || is_expression_useful(&el.value),
            ArrayElement::Value(el) => is_expression_useful(&el.value),
            ArrayElement::Variadic(el) => is_expression_useful(&el.value),
            ArrayElement::Missing(_) => false,
        }),
        _ => true,
    }
}
