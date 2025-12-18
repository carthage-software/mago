use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ArrayElement;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Statement;
use mago_syntax::ast::Variable;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

/// Rule that detects redundant literal guard patterns.
///
/// These patterns:
/// ```php
/// if ($x === null) { return null; }
/// return $x;
///
/// if ($x === null) { return null; } else { return $x; }
///
/// if ($x === null) { return null; } elseif ($x === '') { return ''; }
/// return $x;
/// ```
///
/// Can all be simplified to:
/// ```php
/// return $x;
/// ```
#[derive(Debug, Clone)]
pub struct NoRedundantLiteralReturnRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantLiteralReturnConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantLiteralReturnConfig {
    pub level: Level,
}

impl Default for NoRedundantLiteralReturnConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoRedundantLiteralReturnConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantLiteralReturnRule {
    type Config = NoRedundantLiteralReturnConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Literal Return",
            code: "no-redundant-literal-return",
            description: indoc! {"
                Detects redundant literal guard patterns where an if statement checks if a variable
                equals a literal and returns that same literal, followed by returning the variable.

                This pattern is redundant because if the variable equals the literal, returning the
                variable would return the same value anyway.

                This includes patterns with else clauses and elseif chains where all branches
                follow the same redundant pattern.
            "},
            good_example: indoc! {r"
                <?php

                function getValue($x) {
                    return $x;
                }

                function getValueOrDefault($x, $default) {
                    if ($x === null) {
                        return $default;
                    }
                    return $x;
                }
            "},
            bad_example: indoc! {r"
                <?php

                function getValue($x) {
                    if ($x === null) {
                        return null;
                    }
                    return $x;
                }

                function getWithElse($x) {
                    if ($x === null) {
                        return null;
                    } else {
                        return $x;
                    }
                }

                function getWithElseIf($x) {
                    if ($x === null) {
                        return null;
                    } elseif ($x === '') {
                        return '';
                    }
                    return $x;
                }
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Program,
            NodeKind::Block,
            NodeKind::Namespace,
            NodeKind::DeclareColonDelimitedBody,
            NodeKind::ForeachColonDelimitedBody,
            NodeKind::WhileColonDelimitedBody,
            NodeKind::ForColonDelimitedBody,
            NodeKind::IfColonDelimitedBody,
            NodeKind::IfColonDelimitedBodyElseIfClause,
            NodeKind::IfColonDelimitedBodyElseClause,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let statements = match node {
            Node::Program(program) => program.statements.as_slice(),
            Node::Block(block) => block.statements.as_slice(),
            Node::Namespace(namespace) => namespace.statements().as_slice(),
            Node::DeclareColonDelimitedBody(body) => body.statements.as_slice(),
            Node::ForeachColonDelimitedBody(body) => body.statements.as_slice(),
            Node::WhileColonDelimitedBody(body) => body.statements.as_slice(),
            Node::ForColonDelimitedBody(body) => body.statements.as_slice(),
            Node::IfColonDelimitedBody(body) => body.statements.as_slice(),
            Node::IfColonDelimitedBodyElseIfClause(body) => body.statements.as_slice(),
            Node::IfColonDelimitedBodyElseClause(body) => body.statements.as_slice(),
            _ => return,
        };

        for (i, stmt) in statements.iter().enumerate() {
            let Statement::If(if_stmt) = stmt else {
                continue;
            };

            let Some((var_name, main_literal)) = extract_var_literal_check(if_stmt.condition) else {
                continue;
            };

            let body_statements = get_inner_statements(if_stmt.body.statements());
            if !is_return_expression(body_statements, main_literal) {
                continue;
            }

            let mut all_elseif_valid = true;
            for (elseif_condition, elseif_statements) in if_stmt.body.else_if_clauses() {
                let Some((elseif_var, elseif_literal)) = extract_var_literal_check(elseif_condition) else {
                    all_elseif_valid = false;
                    break;
                };

                if elseif_var != var_name {
                    all_elseif_valid = false;
                    break;
                }

                let inner = get_inner_statements(elseif_statements);
                if !is_return_expression(inner, elseif_literal) {
                    all_elseif_valid = false;
                    break;
                }
            }

            if !all_elseif_valid {
                continue;
            }

            let (return_span, delete_includes_return) = if let Some(else_statements) = if_stmt.body.else_statements() {
                let inner = get_inner_statements(else_statements);
                if !is_return_variable(inner, var_name) {
                    continue;
                }
                (if_stmt.span(), true)
            } else {
                let next_stmt = statements.get(i + 1);
                let Some(Statement::Return(return_stmt)) = next_stmt else {
                    continue;
                };

                let Some(return_value) = &return_stmt.value else {
                    continue;
                };

                let Expression::Variable(Variable::Direct(return_var)) = return_value else {
                    continue;
                };

                if return_var.name != var_name {
                    continue;
                }

                (if_stmt.span(), false)
            };

            let issue = Issue::new(self.cfg.level(), "Redundant literal guard: the if statement can be removed.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(if_stmt.span()).with_message("This if statement is redundant"))
                .with_note(format!(
                    "If `{}` equals the literal, returning `{}` would return that literal anyway.",
                    var_name, var_name
                ))
                .with_help("Remove the if statement and just return the variable.");

            ctx.collector.propose(issue, |plan| {
                if delete_includes_return {
                    let replacement = format!("return {};", var_name);
                    plan.replace(
                        return_span.start.offset..return_span.end.offset,
                        replacement,
                        SafetyClassification::Safe,
                    );
                } else {
                    let if_span = if_stmt.span();
                    let delete_end = find_next_non_whitespace(ctx.source_file.contents.as_bytes(), if_span.end.offset);
                    plan.delete(if_span.start.offset..delete_end, SafetyClassification::Safe);
                }
            });
        }
    }
}

/// Extract variable name and literal from a condition like `$var === literal` or `literal === $var`.
fn extract_var_literal_check<'a>(condition: &'a Expression<'a>) -> Option<(&'a str, &'a Expression<'a>)> {
    let Expression::Binary(binary) = condition else {
        return None;
    };

    let BinaryOperator::Identical(_) = binary.operator else {
        return None;
    };

    match (binary.lhs, binary.rhs) {
        (Expression::Variable(Variable::Direct(var)), lit) if is_literal(lit) => Some((var.name, lit)),
        (lit, Expression::Variable(Variable::Direct(var))) if is_literal(lit) => Some((var.name, lit)),
        _ => None,
    }
}

/// Get the inner statements, unwrapping a Block if present.
fn get_inner_statements<'a>(statements: &'a [Statement<'a>]) -> &'a [Statement<'a>] {
    if statements.len() == 1
        && let Statement::Block(block) = &statements[0]
    {
        return block.statements.as_slice();
    }
    statements
}

/// Check if statements consist of only `return <expr>;` where expr matches the given expression.
fn is_return_expression<'a>(statements: &[Statement<'a>], expected: &Expression<'a>) -> bool {
    if statements.len() != 1 {
        return false;
    }

    let Statement::Return(return_stmt) = &statements[0] else {
        return false;
    };

    let Some(return_value) = &return_stmt.value else {
        return false;
    };

    expressions_are_equivalent(expected, return_value)
}

/// Check if statements consist of only `return $var;` where var matches the given name.
fn is_return_variable(statements: &[Statement<'_>], var_name: &str) -> bool {
    if statements.len() != 1 {
        return false;
    }

    let Statement::Return(return_stmt) = &statements[0] else {
        return false;
    };

    let Some(return_value) = &return_stmt.value else {
        return false;
    };

    let Expression::Variable(Variable::Direct(var)) = return_value else {
        return false;
    };

    var.name == var_name
}

/// Check if an expression is a literal (null, true, false, int, float, string, array).
fn is_literal(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Literal(_) => true,
        Expression::Array(arr) => arr.elements.iter().all(is_array_element_literal),
        Expression::LegacyArray(arr) => arr.elements.iter().all(is_array_element_literal),
        _ => false,
    }
}

/// Check if an array element is a literal.
fn is_array_element_literal(element: &ArrayElement<'_>) -> bool {
    match element {
        ArrayElement::KeyValue(kv) => is_literal(kv.key) && is_literal(kv.value),
        ArrayElement::Value(v) => is_literal(v.value),
        ArrayElement::Variadic(_) => false,
        ArrayElement::Missing(_) => true,
    }
}

/// Check if two expressions are equivalent.
fn expressions_are_equivalent(a: &Expression<'_>, b: &Expression<'_>) -> bool {
    match (a, b) {
        (Expression::Literal(lit_a), Expression::Literal(lit_b)) => literals_are_equivalent(lit_a, lit_b),
        (Expression::Array(arr_a), Expression::Array(arr_b)) => {
            arrays_are_equivalent(arr_a.elements.as_slice(), arr_b.elements.as_slice())
        }
        (Expression::LegacyArray(arr_a), Expression::LegacyArray(arr_b)) => {
            arrays_are_equivalent(arr_a.elements.as_slice(), arr_b.elements.as_slice())
        }
        (Expression::Array(arr_a), Expression::LegacyArray(arr_b))
        | (Expression::LegacyArray(arr_b), Expression::Array(arr_a)) => {
            arrays_are_equivalent(arr_a.elements.as_slice(), arr_b.elements.as_slice())
        }
        _ => false,
    }
}

/// Check if two literals are equivalent.
fn literals_are_equivalent(a: &Literal<'_>, b: &Literal<'_>) -> bool {
    match (a, b) {
        (Literal::Null(_), Literal::Null(_)) => true,
        (Literal::True(_), Literal::True(_)) => true,
        (Literal::False(_), Literal::False(_)) => true,
        (Literal::Integer(a), Literal::Integer(b)) => a.value == b.value,
        (Literal::Float(a), Literal::Float(b)) => a.value == b.value,
        (Literal::String(a), Literal::String(b)) => a.value == b.value,
        _ => false,
    }
}

/// Check if two array element sequences are equivalent.
fn arrays_are_equivalent(a: &[ArrayElement<'_>], b: &[ArrayElement<'_>]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.iter().zip(b.iter()).all(|(elem_a, elem_b)| array_elements_are_equivalent(elem_a, elem_b))
}

/// Check if two array elements are equivalent.
fn array_elements_are_equivalent(a: &ArrayElement<'_>, b: &ArrayElement<'_>) -> bool {
    match (a, b) {
        (ArrayElement::KeyValue(kv_a), ArrayElement::KeyValue(kv_b)) => {
            expressions_are_equivalent(kv_a.key, kv_b.key) && expressions_are_equivalent(kv_a.value, kv_b.value)
        }
        (ArrayElement::Value(v_a), ArrayElement::Value(v_b)) => expressions_are_equivalent(v_a.value, v_b.value),
        (ArrayElement::Missing(_), ArrayElement::Missing(_)) => true,
        _ => false,
    }
}

/// Find the next non-whitespace character position after the given offset.
fn find_next_non_whitespace(bytes: &[u8], offset: u32) -> u32 {
    let mut pos = offset as usize;
    while pos < bytes.len() {
        match bytes[pos] {
            b' ' | b'\t' | b'\r' | b'\n' => pos += 1,
            _ => break,
        }
    }
    pos as u32
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoRedundantLiteralReturnRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_different_return_values,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === null) {
                    return 'default';
                }
                return $x;
            }
        "}
    }

    test_lint_success! {
        name = valid_else_returns_different_value,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === null) {
                    return null;
                } else {
                    return 'something else';
                }
            }
        "}
    }

    test_lint_success! {
        name = valid_elseif_different_var,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x, $y) {
                if ($x === null) {
                    return null;
                } elseif ($y === '') {
                    return '';
                }
                return $x;
            }
        "}
    }

    test_lint_success! {
        name = valid_loose_equality,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x == null) {
                    return null;
                }
                return $x;
            }
        "}
    }

    test_lint_success! {
        name = valid_extra_statement_in_body,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === null) {
                    log('returning null');
                    return null;
                }
                return $x;
            }
        "}
    }

    test_lint_success! {
        name = valid_different_variables,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x, $y) {
                if ($x === null) {
                    return null;
                }
                return $y;
            }
        "}
    }

    test_lint_success! {
        name = valid_no_return_after_if,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === null) {
                    return null;
                }
                echo $x;
            }
        "}
    }

    test_lint_success! {
        name = valid_elseif_returns_different_literal,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === null) {
                    return null;
                } elseif ($x === '') {
                    return 'empty';
                }
                return $x;
            }
        "}
    }

    test_lint_failure! {
        name = null_check,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === null) {
                    return null;
                }
                return $x;
            }
        "}
    }

    test_lint_failure! {
        name = null_check_reversed,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if (null === $x) {
                    return null;
                }
                return $x;
            }
        "}
    }

    test_lint_failure! {
        name = true_check,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === true) {
                    return true;
                }
                return $x;
            }
        "}
    }

    test_lint_failure! {
        name = false_check,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === false) {
                    return false;
                }
                return $x;
            }
        "}
    }

    test_lint_failure! {
        name = integer_check,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === 0) {
                    return 0;
                }
                return $x;
            }
        "}
    }

    test_lint_failure! {
        name = string_check,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === 'default') {
                    return 'default';
                }
                return $x;
            }
        "}
    }

    test_lint_failure! {
        name = empty_array_check,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === []) {
                    return [];
                }
                return $x;
            }
        "}
    }

    test_lint_failure! {
        name = empty_legacy_array_check,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === array()) {
                    return array();
                }
                return $x;
            }
        "}
    }

    test_lint_failure! {
        name = non_empty_array_check,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === [1, 2, 3]) {
                    return [1, 2, 3];
                }
                return $x;
            }
        "}
    }

    test_lint_failure! {
        name = with_else_clause,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === null) {
                    return null;
                } else {
                    return $x;
                }
            }
        "}
    }

    test_lint_failure! {
        name = with_elseif_and_return,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === null) {
                    return null;
                } elseif ($x === '') {
                    return '';
                }
                return $x;
            }
        "}
    }

    test_lint_failure! {
        name = with_elseif_and_else,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === null) {
                    return null;
                } elseif ($x === '') {
                    return '';
                } else {
                    return $x;
                }
            }
        "}
    }

    test_lint_failure! {
        name = multiple_elseif_clauses,
        rule = NoRedundantLiteralReturnRule,
        code = indoc! {r"
            <?php

            function getValue($x) {
                if ($x === null) {
                    return null;
                } elseif ($x === '') {
                    return '';
                } elseif ($x === 0) {
                    return 0;
                }
                return $x;
            }
        "}
    }
}
